use ibc_client_tendermint::types::ClientState as TmClientState;
use ibc_core_client_types::error::UpgradeClientError;
use ibc_core_host_types::path::UpgradeClientPath;
use ibc_primitives::prelude::*;
use tendermint::abci::Event as TmEvent;

use crate::upgrade_proposal::{UpgradeClientProposal, UpgradeExecutionContext, UpgradeProposal};

/// Handles an upgrade client proposal
///
/// It clears both IBC client and consensus states if a previous plan was set.
/// Then it will schedule an upgrade and finally set the upgraded client state
/// in upgrade store.
pub fn upgrade_client_proposal_handler<Ctx>(
    ctx: &mut Ctx,
    proposal: UpgradeProposal,
) -> Result<TmEvent, UpgradeClientError>
where
    Ctx: UpgradeExecutionContext,
    Ctx::AnyClientState: From<TmClientState>,
{
    let plan = proposal.plan;

    if ctx.upgrade_plan().is_ok() {
        ctx.clear_upgrade_plan(plan.height)?;
    }

    let mut client_state =
        TmClientState::try_from(proposal.upgraded_client_state).map_err(|e| {
            UpgradeClientError::InvalidUpgradeProposal {
                reason: e.to_string(),
            }
        })?;

    client_state.zero_custom_fields();

    ctx.schedule_upgrade(plan.clone())?;

    let upgraded_client_state_path = UpgradeClientPath::UpgradedClientState(plan.height);

    ctx.store_upgraded_client_state(upgraded_client_state_path, client_state.into())?;

    let event = TmEvent::from(UpgradeClientProposal::new(proposal.title, plan.height));

    Ok(event)
}
