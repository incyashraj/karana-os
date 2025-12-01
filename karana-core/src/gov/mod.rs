use alloy_primitives::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Default)]
pub struct KaraToken {
    balances: HashMap<String, U256>,  // Addr → Balance
    pub total_supply: U256,
}

impl KaraToken {
    pub fn mint(&mut self, addr: &str, amount: U256) {
        *self.balances.entry(addr.to_string()).or_insert(U256::ZERO) += amount;
        self.total_supply += amount;
    }

    pub fn balance_of(&self, addr: &str) -> U256 {
        *self.balances.get(addr).unwrap_or(&U256::ZERO)
    }

    pub fn transfer(&mut self, from: &str, to: &str, amount: U256) -> bool {
        if self.balance_of(from) < amount { return false; }
        *self.balances.entry(from.to_string()).or_insert(U256::ZERO) -= amount;
        *self.balances.entry(to.to_string()).or_insert(U256::ZERO) += amount;
        true
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub yes_votes: U256,
    pub no_votes: U256,
    pub quorum: U256,  // Min stake for valid
}

#[derive(Default)]
pub struct KaranaDAO {
    pub proposals: HashMap<u32, Proposal>,
    pub token: KaraToken,
    next_id: u32,
}

impl KaranaDAO {
    pub fn propose(&mut self, title: &str, desc: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let prop = Proposal {
            id,
            title: title.to_string(),
            description: desc.to_string(),
            yes_votes: U256::ZERO,
            no_votes: U256::ZERO,
            quorum: U256::from(100u64),  // 100 KARA min
        };
        self.proposals.insert(id, prop);
        id
    }

    pub fn vote(&mut self, addr: &str, proposal_id: u32, yes: bool) -> Result<bool> {
        let stake = self.token.balance_of(addr);
        // Simple check: Must have stake >= quorum to vote? Or just weight?
        // Prompt says: "quorum: U256 // Min stake for valid" usually means min total votes.
        // But code says: if stake < quorum { return Ok(false); }
        // This implies a minimum stake requirement to participate.
        if stake < self.proposals.get(&proposal_id).unwrap().quorum { return Ok(false); }

        let prop = self.proposals.get_mut(&proposal_id).ok_or(anyhow::anyhow!("No proposal"))?;
        if yes {
            prop.yes_votes += stake;
        } else {
            prop.no_votes += stake;
        }

        // Tally: Passed if yes > no + quorum
        // This logic seems to imply quorum is a bias against passing? 
        // Or maybe it means yes must exceed no by at least quorum?
        // Prompt code: let passed = prop.yes_votes > prop.no_votes + prop.quorum;
        let passed = prop.yes_votes > prop.no_votes + prop.quorum;
        Ok(passed)
    }

    pub fn execute_if_passed(&self, id: u32, executor: &mut dyn FnMut(u32)) {
        if let Some(prop) = self.proposals.get(&id) {
            if prop.yes_votes > prop.no_votes + prop.quorum {
                executor(id);  // e.g., Runtime ignite new feature
            }
        }
    }

    pub fn propose_bounty(&mut self, bug_proof: &str, severity: u8) -> u32 {
        let amount = U256::from(10u64 * severity as u64);  // 10-50 KARA
        let id = self.propose(&format!("Bounty: {}", bug_proof), "Claim if fixed");
        // Auto-mint to treasury (simulated)
        self.token.mint("treasury", amount);
        id
    }

    pub fn claim_bounty(&mut self, addr: &str, id: u32) -> Result<U256> {
        // In a real system, this would check if the proposal passed
        // Here we simulate the vote passing for the claim
        if self.vote(addr, id, true).unwrap() {
             // Fixed bounty amount for now, or retrieve from proposal metadata if we stored it
            let bounty = U256::from(50u64); 
            if self.token.transfer("treasury", addr, bounty) {
                Ok(bounty)
            } else {
                // Mint if treasury empty (simulated inflation)
                self.token.mint(addr, bounty);
                Ok(bounty)
            }
        } else { 
            Err(anyhow::anyhow!("Not passed")) 
        }
    }
}
