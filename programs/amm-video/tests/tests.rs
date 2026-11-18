use {
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

use anchor_lang::solana_program::program_pack::Pack;
use litesvm_token::spl_token::{
    state::{Account as SplTokenAccount, Mint as SplMint},
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

// Setup function to initialize LiteSVM and create a payer keypair
fn setup() -> (
    LiteSVM,
    Keypair,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = amm_video::id();
    let payer = Keypair::new();
    let treasury = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/amm_video.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // Create two mints (Mint A and Mint B) with 6 decimal places and the maker as the authority
    // This done using litesvm-token's CreateMint utility which creates the mint in the LiteSVM environment
    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();
    
    let (mint_x, mint_y) = if mint_x > mint_y {
        (mint_x, mint_y)
    } else {
        (mint_y, mint_x)
    };

    let config =
        Pubkey::find_program_address(&[b"config", &123u64.to_le_bytes()], &amm_video::id()).0;
    let mint_lp = Pubkey::find_program_address(&[b"lp", config.as_ref()], &amm_video::id()).0;

    // Derive the PDA for the vault associated token account using the config PDA and Mint A
    let vault_x = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y = associated_token::get_associated_token_address(&config, &mint_y);

    (
        svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    )
}

fn get_token_account( svm: &LiteSVM, address: Pubkey,) -> SplTokenAccount {
    let account = svm
        .get_account(&address)
        .expect("Token account not found");

    SplTokenAccount::unpack(&account.data)
        .expect("Failed to deserialize token account")
}

fn get_mint(svm: &LiteSVM, address: Pubkey ) -> SplMint {
    let account = svm
        .get_account(&address)
        .expect("Mint account not found");

    SplMint::unpack(&account.data)
        .expect("Failed to deserialize mint")
}

#[test]
fn test_initialize() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let instruction = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());
    println!("Transaction result: {:#?}", res);
    assert!(res.is_ok());
}

#[test]
pub fn test_deposit() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());
    println!("Transaction result: {:#?}", res);
    assert!(res.is_ok());
}

#[test]
pub fn test_withdraw() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let withdraw_ix = create_withdraw_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let res = send(
        &mut svm,
        &[init_ix, deposit_ix, withdraw_ix],
        &payer,
        &[&payer],
    );
    assert!(res.is_ok());
}

#[test]
pub fn test_swap() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let swap_ix = create_swap_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, mint_lp, config, vault_x, vault_y,
        true, 10_000_000, 5_000_000,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());
    println!("Transaction result: {:#?}", res);
    assert!(res.is_ok());
}


#[test]
fn test_update_fee() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let update_ix = create_update_fee_ix(&payer, config, 75);
    let res = send(&mut svm, &[init_ix, update_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    if let Err(err) = &res {
        println!("Transaction error: {:#?}", err);
        println!("Execution logs:\n{}", err.meta.logs.join("\n"));
    }

    assert!(res.is_ok());
}

#[test]
fn test_update_fee_rejects_fee_at_or_above_100_percent() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let update_ix = create_update_fee_ix(&payer, config, 10_000);
    let res = send(&mut svm, &[init_ix, update_ix], &payer, &[&payer]);
    assert!(res.is_err());
}

#[test]
fn test_treasury_withdraws_collected_swap_fees() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, mint_lp, config, vault_x, vault_y,
        true, 10_000_000, 5_000_000,
    );
    let swap_result = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(swap_result.is_ok());

    // 0.30% of 10,000,000 input = 30,000 units.
    let withdraw_ix = create_withdraw_treasury_ix(
        &mut svm, &treasury, payer.pubkey(), mint_x, mint_y, config, 30_000,
    );
    let withdraw_result = send(
        &mut svm,
        &[withdraw_ix],
        &payer,
        &[&payer, &treasury],
    );
    if let Err(err) = &withdraw_result {
        println!("Treasury withdrawal error:\n{err:#?}");
        println!("Transaction logs:\n{}", err.meta.logs.join("\n"));
    }

assert!(withdraw_result.is_ok());

    let overdraw_ix = create_withdraw_treasury_ix(
        &mut svm, &treasury, payer.pubkey(), mint_x, mint_y, config, 1,
    );
    let overdraw_result = send(
        &mut svm,
        &[overdraw_ix],
        &payer,
        &[&payer, &treasury],
    );
    assert!(overdraw_result.is_err());
}


#[test]
fn test_swap_y_for_x() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, mint_lp, config, vault_x, vault_y,
        false, 10_000_000, 5_000_000,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());
    if let Err(err) = &res {
        println!("Transaction error: {:#?}", err);
        println!("Execution logs:\n{}", err.meta.logs.join("\n"));
    }

    assert!(res.is_ok());
}

#[test]
fn test_swap_rejects_excessive_minimum_output() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, mint_lp, config, vault_x, vault_y,
        true, 10_000_000, 20_000_000,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_err());
    println!("Transaction result: {:#?}", res);
    let err = res.unwrap_err();
    assert!(err.meta.logs.join("\n").contains("Error Code: SlippageExceeded"));
}


#[test]
fn test_update_fee_rejects_non_authority() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let unauthorized = Keypair::new();
    let update_ix = create_update_fee_ix(&unauthorized, config, 100);
    let res = send(
        &mut svm,
        &[init_ix, update_ix],
        &payer,
        &[&payer, &unauthorized],
    );
    assert!(res.is_err());
}


#[test]
fn test_treasury_burns_collected_swap_fees() {
    let (mut svm, payer, treasury, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    // The treasury token accounts only exist after a swap, so create the pool and swap first.
    let init_ix = create_initialise_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_ix(
        &mut svm, &payer, treasury.pubkey(), mint_x, mint_y, mint_lp, config, vault_x, vault_y,
        true, 10_000_000, 5_000_000,
    );
    send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]).unwrap();

    let treasury_x = associated_token::get_associated_token_address(&treasury.pubkey(), &mint_x);
    let amount_to_burn = 10_000;

    let treasury_x_before = get_token_account(&svm, treasury_x);
    let mint_x_before = get_mint(&svm, mint_x);
    assert_eq!(treasury_x_before.amount, 30_000); // 0.30% of 10,000,000

    let burn_ix = create_burn_treasury_ix(
        &mut svm, &treasury, payer.pubkey(), amount_to_burn, mint_x, mint_y, mint_x, config,
    );
    // the treasury wallet holds no SOL, so the payer covers the fee and the treasury co-signs
    let burn_result = send(&mut svm, &[burn_ix], &payer, &[&payer, &treasury]);
    if let Err(err) = &burn_result {
        println!("{}", err.meta.logs.join("\n"));
    }
    assert!(burn_result.is_ok());

    let treasury_x_after = get_token_account(&svm, treasury_x);
    let mint_x_after = get_mint(&svm, mint_x);
    assert_eq!(treasury_x_before.amount - treasury_x_after.amount, amount_to_burn);
    assert_eq!(mint_x_before.supply - mint_x_after.supply, amount_to_burn);
}