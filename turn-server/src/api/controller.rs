use std::{net::SocketAddr, sync::Arc};

use super::payload::{Node, Stats, SOFTWARE};
use crate::{config::*, server::*};

use axum::{extract::Query, extract::State, Json};
use serde::Deserialize;
use tokio::time::Instant;
use turn_rs::Service;

#[derive(Debug, Deserialize)]
pub struct AddrParams {
    addr: SocketAddr,
}

#[derive(Debug, Deserialize)]
pub struct Qiter {
    skip: Option<usize>,
    limit: Option<usize>,
}

/// controller
///
/// It is possible to control the turn server and obtain server internal
/// information and reports through the controller.
pub struct Controller {
    config: Arc<Config>,
    service: Service,
    monitor: Monitor,
    timer: Instant,
}

impl Controller {
    /// Create a controller.
    ///
    /// Controllers require external routing and thread monitoring instances, as
    /// well as configuration information.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::new()
    /// let service = Service::new(/* ... */);;
    /// let monitor = Monitor::new(/* ... */);
    ///
    /// Controller::new(service.get_router(), config, monitor);
    /// ```
    pub fn new(config: Arc<Config>, monitor: Monitor, service: Service) -> Arc<Self> {
        Arc::new(Self {
            timer: Instant::now(),
            monitor,
            service,
            config,
        })
    }

    /// get server status.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::new()
    /// let service = Service::new(/* ... */);;
    /// let monitor = Monitor::new(/* ... */);
    ///
    /// let ctr = Controller::new(service.get_router(), config, monitor);
    /// // let state_js = ctr.get_stats().await;
    /// ```
    pub async fn get_stats(State(this): State<&Self>) -> Json<Stats> {
        let router = this.service.get_router();
        Json(Stats {
            software: SOFTWARE.to_string(),
            uptime: this.timer.elapsed().as_secs(),
            realm: this.config.turn.realm.clone(),
            port_allocated: router.len() as u16,
            port_capacity: router.capacity() as u16,
            interfaces: this.config.turn.interfaces.clone(),
        })
    }

    /// Get a list of sockets
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::new()
    /// let service = Service::new(/* ... */);;
    /// let monitor = Monitor::new(/* ... */);
    ///
    /// let ctr = Controller::new(service.get_router(), config, monitor);
    /// // let workers_js = ctr.get_report().await;
    /// ```
    pub async fn get_report(
        State(this): State<&Self>,
        Query(pars): Query<Qiter>,
    ) -> Json<Vec<(SocketAddr, Store)>> {
        let skip = pars.skip.unwrap_or(0);
        let limit = pars.limit.unwrap_or(20);
        Json(this.monitor.get_nodes(skip, limit))
    }

    /// Get user list.
    ///
    /// This interface returns the username and a list of addresses used by this
    /// user.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::new()
    /// let service = Service::new(/* ... */);;
    /// let monitor = Monitor::new(/* ... */);
    ///
    /// let ctr = Controller::new(service.get_router(), config, monitor);
    /// // let users_js = ctr.get_users().await;
    /// ```
    pub async fn get_users(
        State(this): State<&Self>,
        Query(pars): Query<Qiter>,
    ) -> Json<Vec<(String, Vec<SocketAddr>)>> {
        let router = this.service.get_router();
        let skip = pars.skip.unwrap_or(0);
        let limit = pars.limit.unwrap_or(20);
        Json(router.get_users(skip, limit))
    }

    /// Get node information
    ///
    /// This interface can obtain the user's basic information and assigned
    /// information, including the survival time.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::new()
    /// let service = Service::new(/* ... */);;
    /// let monitor = Monitor::new(/* ... */);
    ///
    /// let ctr = Controller::new(service.get_router(), config, monitor);
    /// let addr = "127.0.0.1:8080".parse().unwrap();
    /// // let user_js = ctr.get_node(addr).await;
    /// ```
    pub async fn get_node(
        State(this): State<&Self>,
        Query(pars): Query<AddrParams>,
    ) -> Json<Option<Node>> {
        let router = this.service.get_router();
        Json(router.get_node(&Arc::new(pars.addr)).map(Node::from))
    }

    /// Delete a node under the user.
    ///
    /// This will cause all information of the current node to be deleted,
    /// including the binding relationship, and at the same time terminate the
    /// INodeion of the current node and stop forwarding data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = Config::new()
    /// let service = Service::new(/* ... */);;
    /// let monitor = Monitor::new(/* ... */);
    ///
    /// let ctr = Controller::new(service.get_router(), config, monitor);
    /// let addr = "127.0.0.1:8080".parse().unwrap();
    /// // let remove_node_js = ctr.remove_user(addr).await;
    /// ```
    pub async fn remove_node(
        State(this): State<&Self>,
        Query(pars): Query<AddrParams>,
    ) -> Json<bool> {
        let router = this.service.get_router();
        Json(router.remove(&Arc::new(pars.addr)).is_some())
    }
}
