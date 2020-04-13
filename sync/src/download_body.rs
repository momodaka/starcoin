use crate::download::Downloader;
use crate::helper::send_sync_request;
use actix::prelude::*;
use anyhow::Result;
use network::NetworkAsyncService;
use network_p2p_api::sync_messages::{DataType, GetDataByHashMsg, ProcessMessage};
use network_p2p_api::sync_messages::{SyncRpcRequest, SyncRpcResponse};
use std::sync::Arc;
use traits::Consensus;
use types::{block::BlockHeader, peer_info::PeerInfo};

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct SyncBodyEvent {
    pub headers: Vec<BlockHeader>,
    pub peers: Vec<PeerInfo>,
}

#[derive(Clone)]
pub struct DownloadBodyActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    downloader: Arc<Downloader<C>>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService,
}

impl<C> DownloadBodyActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn _launch(
        downloader: Arc<Downloader<C>>,
        peer_info: Arc<PeerInfo>,
        network: NetworkAsyncService,
    ) -> Result<Addr<DownloadBodyActor<C>>> {
        Ok(Actor::create(move |_ctx| DownloadBodyActor {
            downloader,
            peer_info,
            network,
        }))
    }
}

impl<C> Actor for DownloadBodyActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;
}

impl<C> Handler<SyncBodyEvent> for DownloadBodyActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;
    fn handle(&mut self, event: SyncBodyEvent, _ctx: &mut Self::Context) -> Self::Result {
        let hashs = event.headers.iter().map(|h| h.id().clone()).collect();
        let get_data_by_hash_msg = GetDataByHashMsg {
            hashs,
            data_type: DataType::BODY,
        };

        let get_data_by_hash_req = SyncRpcRequest::GetDataByHashMsg(
            ProcessMessage::GetDataByHashMsg(get_data_by_hash_msg),
        );

        let network = self.network.clone();
        let peers = event.peers.clone();
        let downloader = self.downloader.clone();

        let headers = event.headers;
        Arbiter::spawn(async move {
            for peer in peers {
                if let SyncRpcResponse::BatchHeaderAndBodyMsg(_, bodies, infos) = send_sync_request(
                    &network,
                    peer.get_peer_id().clone(),
                    get_data_by_hash_req.clone(),
                )
                .await
                .unwrap()
                {
                    Downloader::do_blocks(downloader, headers, bodies.bodies, infos.infos).await;
                    break;
                };
            }
        });

        Ok(())
    }
}
