use bincode::{Decode, Encode};
use std::collections::*;

#[derive(Decode, Encode, Debug, Clone, PartialEq)]
pub struct Signature {
    pub root: Option<[u8; 32]>,
    pub local_timestamp: u128,
}

pub struct MajorityTracker {
    history: HashMap<String, Signature>,
}

impl MajorityTracker {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    pub fn update_signature(&mut self, peer_id: String, new_signature: Signature) {
        if let Some(old) = self.history.get(&peer_id) {
            if old.local_timestamp < new_signature.local_timestamp {
                self.history.insert(peer_id, new_signature);
            }
        } else {
            self.history.insert(peer_id, new_signature);
        }
    }

    fn most_common_root(&self) -> Option<[u8; 32]> {
        let mut freqs = HashMap::new();
        for signature in self.history.values() {
            if let Some(root) = signature.root {
                *freqs.entry(root).or_insert(0) += 1;
            }
        }
        freqs
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(root, _)| root)
    }

    pub fn truthful_majority(&self) -> Option<Vec<String>> {
        if let Some(mc_root) = self.most_common_root() {
            let mut result = Vec::new();
            for (peer_id, signature) in self.history.iter() {
                if signature.root == Some(mc_root) {
                    result.push(peer_id.to_string());
                }
            }
            return Some(result);
        }
        None
    }
}

impl Default for MajorityTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(root: Option<[u8; 32]>, ts: u128) -> Signature {
        Signature {
            root,
            local_timestamp: ts,
        }
    }

    #[test]
    fn test_new_tracker_is_empty() {
        let tracker = MajorityTracker::new();
        assert!(tracker.history.is_empty());
    }

    #[test]
    fn test_update_inserts_new_peer() {
        let mut tracker = MajorityTracker::new();
        tracker.update_signature("p1".to_string(), sig(Some([1; 32]), 100));
        assert_eq!(tracker.history.len(), 1);
    }

    #[test]
    fn test_update_replaces_only_newer_timestamp() {
        let mut tracker = MajorityTracker::new();
        let old = sig(Some([1; 32]), 100);
        let new = sig(Some([2; 32]), 200);

        tracker.update_signature("p1".to_string(), old);
        tracker.update_signature("p1".to_string(), new.clone());

        assert_eq!(tracker.history["p1"], new);
    }

    #[test]
    fn test_update_ignores_older_timestamp() {
        let mut tracker = MajorityTracker::new();
        tracker.update_signature("p1".to_string(), sig(Some([1; 32]), 200));
        tracker.update_signature("p1".to_string(), sig(Some([9; 32]), 100));

        assert_eq!(tracker.history["p1"].local_timestamp, 200);
        assert_eq!(tracker.history["p1"].root, Some([1; 32]));
    }

    #[test]
    fn test_most_common_root_empty() {
        let tracker = MajorityTracker::new();
        assert_eq!(tracker.most_common_root(), None);
    }

    #[test]
    fn test_most_common_root_single() {
        let mut t = MajorityTracker::new();
        let root = [42; 32];
        t.update_signature("p1".to_string(), sig(Some(root), 1));
        assert_eq!(t.most_common_root(), Some(root));
    }

    #[test]
    fn test_most_common_root_majority() {
        let mut t = MajorityTracker::new();
        let a = [1; 32];
        let b = [2; 32];

        t.update_signature("p1".to_string(), sig(Some(a), 1));
        t.update_signature("p2".to_string(), sig(Some(a), 2));
        t.update_signature("p3".to_string(), sig(Some(b), 3));

        assert_eq!(t.most_common_root(), Some(a));
    }

    #[test]
    fn test_most_common_root_tie_returns_one_of_max() {
        let mut t = MajorityTracker::new();
        let a = [1; 32];
        let b = [2; 32];

        t.update_signature("p1".to_string(), sig(Some(a), 1));
        t.update_signature("p2".to_string(), sig(Some(b), 1));

        let mc = t.most_common_root();
        assert!(mc == Some(a) || mc == Some(b));
    }

    #[test]
    fn test_most_common_root_ignores_none() {
        let mut t = MajorityTracker::new();
        let root = [1; 32];

        t.update_signature("p1".to_string(), sig(Some(root), 1));
        t.update_signature("p2".to_string(), sig(None, 2));
        t.update_signature("p3".to_string(), sig(None, 3));

        assert_eq!(t.most_common_root(), Some(root));
    }

    #[test]
    fn test_most_common_root_skips_none_in_frequency() {
        let mut t = MajorityTracker::new();

        t.update_signature("p1".to_string(), sig(None, 1));
        t.update_signature("p2".to_string(), sig(None, 2));
        t.update_signature("p3".to_string(), sig(Some([1; 32]), 3));
    }
}
