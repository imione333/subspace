use crate::node_client::{Error as RpcError, Error, NodeClient};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use jsonrpsee::core::client::{ClientT, SubscriptionClientT};
use jsonrpsee::core::Error as JsonError;
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use std::pin::Pin;
use std::sync::Arc;
use subspace_archiving::archiver::ArchiverSegment;
use subspace_core_primitives::{SegmentCommitment, SegmentHeader, SegmentIndex};
use subspace_rpc_primitives::{
    FarmerAppInfo, RewardSignatureResponse, RewardSigningInfo, SlotInfo, SolutionResponse,
};

// Defines max_concurrent_requests constant in the node rpc client.
// It must be set for large plots.
const WS_PRC_MAX_CONCURRENT_REQUESTS: usize = 1_000_000;

/// `WsClient` wrapper.
#[derive(Clone, Debug)]
pub struct NodeRpcClient {
    client: Arc<WsClient>,
}

impl NodeRpcClient {
    /// Create a new instance of [`NodeClient`].
    pub async fn new(url: &str) -> Result<Self, JsonError> {
        let client = Arc::new(
            WsClientBuilder::default()
                .max_concurrent_requests(WS_PRC_MAX_CONCURRENT_REQUESTS)
                .max_request_body_size(20 * 1024 * 1024)
                .build(url)
                .await?,
        );
        Ok(Self { client })
    }
}

#[async_trait]
impl NodeClient for NodeRpcClient {
    async fn farmer_app_info(&self) -> Result<FarmerAppInfo, Error> {
        Ok(self
            .client
            .request("subspace_getFarmerAppInfo", rpc_params![])
            .await?)
    }

    async fn subscribe_slot_info(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = SlotInfo> + Send + 'static>>, RpcError> {
        let subscription = self
            .client
            .subscribe(
                "subspace_subscribeSlotInfo",
                rpc_params![],
                "subspace_unsubscribeSlotInfo",
            )
            .await?;

        Ok(Box::pin(subscription.filter_map(
            |slot_info_result| async move { slot_info_result.ok() },
        )))
    }

    async fn submit_solution_response(
        &self,
        solution_response: SolutionResponse,
    ) -> Result<(), RpcError> {
        Ok(self
            .client
            .request(
                "subspace_submitSolutionResponse",
                rpc_params![&solution_response],
            )
            .await?)
    }

    async fn subscribe_reward_signing(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = RewardSigningInfo> + Send + 'static>>, RpcError> {
        let subscription = self
            .client
            .subscribe(
                "subspace_subscribeRewardSigning",
                rpc_params![],
                "subspace_unsubscribeRewardSigning",
            )
            .await?;

        Ok(Box::pin(subscription.filter_map(
            |reward_signing_info_result| async move { reward_signing_info_result.ok() },
        )))
    }

    /// Submit a block signature
    async fn submit_reward_signature(
        &self,
        reward_signature: RewardSignatureResponse,
    ) -> Result<(), RpcError> {
        Ok(self
            .client
            .request(
                "subspace_submitRewardSignature",
                rpc_params![&reward_signature],
            )
            .await?)
    }

    async fn subscribe_archived_segments(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = ArchiverSegment> + Send + 'static>>, RpcError> {
        let subscription = self
            .client
            .subscribe(
                "subspace_subscribeArchivedSegment",
                rpc_params![],
                "subspace_unsubscribeArchivedSegment",
            )
            .await?;

        Ok(Box::pin(subscription.filter_map(
            |archived_segment_result| async move { archived_segment_result.ok() },
        )))
    }

    async fn segment_commitments(
        &self,
        segment_indexes: Vec<SegmentIndex>,
    ) -> Result<Vec<Option<SegmentCommitment>>, RpcError> {
        Ok(self
            .client
            .request("subspace_recordsRoots", rpc_params![&segment_indexes])
            .await?)
    }

    async fn segment_headers(
        &self,
        segment_indexes: Vec<SegmentIndex>,
    ) -> Result<Vec<Option<SegmentHeader>>, RpcError> {
        Ok(self
            .client
            .request("subspace_SegmentHeaders", rpc_params![&segment_indexes])
            .await?)
    }
}
