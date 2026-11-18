use {
    anchor_lang::{
        solana_program::instruction::Instruction, InstructionData, ToAccountMetas,
    },
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn create_update_fee_ix(
    authority: &Keypair,
    config: Pubkey,
    new_fee: u16,
) -> Instruction {
    Instruction::new_with_bytes(
        amm_video::id(),
        &amm_video::instruction::UpdateFee { new_fee }.data(),
        amm_video::accounts::UpdateFee {
            authority: authority.pubkey(),
            config,
        }
        .to_account_metas(None),
    )
}
