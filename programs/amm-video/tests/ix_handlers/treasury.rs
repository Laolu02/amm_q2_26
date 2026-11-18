use {
    anchor_lang::{
        solana_program::instruction::Instruction, InstructionData, ToAccountMetas,
    },
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn create_withdraw_treasury_ix(
    _svm: &mut LiteSVM,
    treasury: &Keypair,
    recipient: Pubkey,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    amount: u64,
) -> Instruction {
    let treasury_key = treasury.pubkey();
    let recipient_x = associated_token::get_associated_token_address(&recipient, &mint_x);
    let recipient_y = associated_token::get_associated_token_address(&recipient, &mint_y);
    let treasury_x = associated_token::get_associated_token_address(&treasury_key, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury_key, &mint_y);
    let is_x = true;

    Instruction::new_with_bytes(
        amm_video::id(),
        &amm_video::instruction::WithdrawTreasury {
            is_x: true,
            amount,
        }
        .data(),
        amm_video::accounts::Treasury{
            treasury: treasury_key,
            recipient,
            mint_x,
            mint_y,
            config,
            mint: if is_x {mint_x} else {mint_y},
            treasury_x,
            treasury_y,
            recipient_x,
            recipient_y,
            token_program: litesvm_token::spl_token::ID,
        }
        .to_account_metas(None)
    )
}


pub fn create_burn_treasury_ix(
    _svm: &mut LiteSVM,
    treasury: &Keypair,
    recipient: Pubkey,
    amount: u64,
    mint_x: Pubkey,
    mint_y: Pubkey,
    mint: Pubkey,
    config: Pubkey,
) -> Instruction {
    let treasury_key = treasury.pubkey();

    let treasury_x = associated_token::get_associated_token_address(&treasury_key, &mint_x);
    let treasury_y = associated_token::get_associated_token_address (&treasury_key, &mint_y);
    let recipient_x = associated_token::get_associated_token_address(&recipient, &mint_x);
    let recipient_y = associated_token::get_associated_token_address(&recipient, &mint_y);

    Instruction::new_with_bytes(
        amm_video::id(),
        &amm_video::instruction::BurnTreasury { amount }.data(),
        amm_video::accounts::Treasury {
            treasury: treasury_key,
            recipient,
            mint_x,
            mint_y,
            mint,
            config,
            treasury_x,
            treasury_y,
            recipient_x,
            recipient_y,
            token_program: litesvm_token::spl_token::ID,
        }
        .to_account_metas(None),
    )
}