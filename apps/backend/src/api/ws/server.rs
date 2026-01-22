//! WebSocket server message handling - sends messages to clients

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures::SinkExt;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;

use crate::models::api::{OrderbookData, OrderbookStatsData, PriceLevel, ServerMessage};
use crate::models::domain::{EngineEvent, Subscription};

use super::{
    state::SubscriptionSet, SocketState, PING_INTERVAL, PONG_TIMEOUT, UNSUBSCRIBED_TIMEOUT,
};

/// Handle outgoing messages to the client and ping/pong management
pub(super) async fn handle_server_messages(
    mut sender: futures::stream::SplitSink<WebSocket, Message>,
    mut event_rx: broadcast::Receiver<EngineEvent>,
    socket_state: Arc<RwLock<SocketState>>,
    mut ack_rx: tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
) {
    let mut ping_interval = interval(PING_INTERVAL);

    loop {
        tokio::select! {
            // Send ping and check for timeouts
            _ = ping_interval.tick() => {
                let state = socket_state.read().await;

                // 1. Check if last pong was too long ago (dead connection)
                if state.last_pong.elapsed() > PONG_TIMEOUT {
                    log::warn!("No pong received for {:?}, disconnecting client", state.last_pong.elapsed());
                    break;
                }

                // 2. Check if client has no subscriptions for too long
                if state.subscriptions.is_empty() && state.last_subscription_change.elapsed() > UNSUBSCRIBED_TIMEOUT {
                    log::info!("Client has no subscriptions for {:?}, disconnecting", state.last_subscription_change.elapsed());
                    break;
                }

                drop(state); // Release lock

                // 3. Send ping
                if sender.send(Message::Ping(Bytes::new())).await.is_err() {
                    log::error!("Failed to send ping, client disconnected");
                    break;
                }
                log::debug!("Sent ping to client");
            }

            // Send acknowledgment messages
            Some(ack) = ack_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&ack) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        log::error!("Failed to send acknowledgment to client");
                        break;
                    }
                    log::debug!("Sent acknowledgment: {:?}", ack);
                }
            }

            // Forward engine events to client
            Ok(event) = event_rx.recv() => {
                let state = socket_state.read().await;
                let messages = engine_event_to_messages(&event, &state.subscriptions);
                drop(state); // Release lock before serialization

                for server_msg in messages {
                    if let Ok(json) = serde_json::to_string(&server_msg) {
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            log::error!("Failed to send message to client");
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Convert an EngineEvent to ServerMessage(s) for WebSocket transmission
/// Returns multiple messages if the event matches multiple subscription types
fn engine_event_to_messages(
    event: &EngineEvent,
    subscriptions: &SubscriptionSet,
) -> Vec<ServerMessage> {
    let mut messages = Vec::new();

    match event {
        EngineEvent::TradeExecuted { trade } => {
            // Early return if no relevant subscriptions
            if !subscriptions.wants_event(event) {
                return messages;
            }

            let trade_data = crate::models::api::TradeData {
                id: trade.id.to_string(),
                market_id: trade.market_id.clone(),
                buyer_address: trade.buyer_address.clone(),
                seller_address: trade.seller_address.clone(),
                buyer_order_id: trade.buyer_order_id.to_string(),
                seller_order_id: trade.seller_order_id.to_string(),
                price: trade.price.to_string(),
                size: trade.size.to_string(),
                side: trade.side,
                timestamp: trade.timestamp.timestamp(),
            };

            // Send Trade message if subscribed to market-wide trades
            if subscriptions.has_subscription(&Subscription::Trades {
                market_id: trade.market_id.clone(),
            }) {
                messages.push(ServerMessage::Trade {
                    trade: trade_data.clone(),
                });
            }

            // Send UserFill message if subscribed to user fills (buyer or seller)
            if subscriptions.has_subscription(&Subscription::UserFills {
                user_address: trade.buyer_address.clone(),
            }) || subscriptions.has_subscription(&Subscription::UserFills {
                user_address: trade.seller_address.clone(),
            }) {
                messages.push(ServerMessage::UserFill { trade: trade_data });
            }
        }
        EngineEvent::OrderPlaced { order } => {
            if subscriptions.wants_event(event) {
                messages.push(ServerMessage::UserOrder {
                    order_id: order.id.to_string(),
                    status: format!("{:?}", order.status).to_lowercase(),
                    filled_size: order.filled_size.to_string(),
                });
            }
        }
        EngineEvent::OrderCancelled { order_id, .. } => {
            if subscriptions.wants_event(event) {
                messages.push(ServerMessage::UserOrder {
                    order_id: order_id.to_string(),
                    status: "cancelled".to_string(),
                    filled_size: "0".to_string(),
                });
            }
        }
        EngineEvent::BalanceUpdated { balance } => {
            if subscriptions.wants_event(event) {
                messages.push(ServerMessage::UserBalance {
                    user_address: balance.user_address.clone(),
                    token_ticker: balance.token_ticker.clone(),
                    available: balance
                        .amount
                        .saturating_sub(balance.open_interest)
                        .to_string(),
                    locked: balance.open_interest.to_string(),
                    updated_at: balance.updated_at.timestamp(),
                });
            }
        }
        EngineEvent::OrderbookSnapshot { orderbook } => {
            if subscriptions.wants_event(event) {
                messages.push(ServerMessage::Orderbook {
                    orderbook: OrderbookData {
                        market_id: orderbook.market_id.clone(),
                        bids: orderbook
                            .bids
                            .iter()
                            .map(|level| PriceLevel {
                                price: level.price.to_string(),
                                size: level.size.to_string(),
                            })
                            .collect(),
                        asks: orderbook
                            .asks
                            .iter()
                            .map(|level| PriceLevel {
                                price: level.price.to_string(),
                                size: level.size.to_string(),
                            })
                            .collect(),
                        stats: orderbook.stats.as_ref().map(|s| OrderbookStatsData {
                            vwap_bid: s.vwap_bid.clone(),
                            vwap_ask: s.vwap_ask.clone(),
                            spread: s.spread.clone(),
                            spread_bps: s.spread_bps.clone(),
                            micro_price: s.micro_price.clone(),
                            imbalance: s.imbalance,
                            bid_depth: s.bid_depth.clone(),
                            ask_depth: s.ask_depth.clone(),
                        }),
                    },
                });
            }
        }
    }

    messages
}
