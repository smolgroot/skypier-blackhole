use radix_trie::Trie;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::Result;

pub struct BlocklistManager {
    // Domain blocklist using radix trie for efficient lookups
    domains: Arc<RwLock<Trie<String, ()>>>,
    
    // Exact domain matches
    exact_matches: Arc<RwLock<HashSet<String>>>,
}

impl BlocklistManager {
    pub fn new() -> Self {
        BlocklistManager {
            domains: Arc::new(RwLock::new(Trie::new())),
            exact_matches: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    /// Check if a domain is blocked
    pub async fn is_blocked(&self, domain: &str) -> bool {
        // Check exact match first
        let exact = self.exact_matches.read().await;
        if exact.contains(domain) {
            return true;
        }
        
        // Check wildcard matches in trie
        let trie = self.domains.read().await;
        trie.get(domain).is_some()
    }
    
    /// Add a domain to the blocklist
    pub async fn add_domain(&self, domain: String) -> Result<()> {
        let mut exact = self.exact_matches.write().await;
        exact.insert(domain.clone());
        
        let mut trie = self.domains.write().await;
        trie.insert(domain, ());
        
        Ok(())
    }
    
    /// Remove a domain from the blocklist
    pub async fn remove_domain(&self, domain: &str) -> Result<()> {
        let mut exact = self.exact_matches.write().await;
        exact.remove(domain);
        
        let mut trie = self.domains.write().await;
        trie.remove(domain);
        
        Ok(())
    }
    
    /// Load domains from a list
    pub async fn load_domains(&self, domains: Vec<String>) -> Result<()> {
        let mut exact = self.exact_matches.write().await;
        let mut trie = self.domains.write().await;
        
        for domain in domains {
            exact.insert(domain.clone());
            trie.insert(domain, ());
        }
        
        Ok(())
    }
    
    /// Get the number of blocked domains
    pub async fn count(&self) -> usize {
        let exact = self.exact_matches.read().await;
        exact.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blocklist() {
        let manager = BlocklistManager::new();
        
        manager.add_domain("example.com".to_string()).await.unwrap();
        assert!(manager.is_blocked("example.com").await);
        assert!(!manager.is_blocked("other.com").await);
        
        manager.remove_domain("example.com").await.unwrap();
        assert!(!manager.is_blocked("example.com").await);
    }
}
