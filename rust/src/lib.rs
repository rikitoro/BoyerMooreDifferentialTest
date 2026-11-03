/// Boyer-Moore majority vote アルゴリズムの実装
/// 

pub fn majority_candidate (votes: &[u32]) -> u32 {
    let mut candidate: u32 = 0;
    let mut count: usize = 0;

    for &vote in votes {
        if count == 0 {
            candidate = vote;
            count = 1;
        } else if candidate == vote {
            count += 1;
        } else {
            count -= 1;
        }
    }
    candidate
}


#[cfg(test)]
mod tests {
    use super::majority_candidate;

    #[test]
    fn empty_returns_zero() {
        assert_eq!(majority_candidate(&[]), 0);
    }

    #[test]
    fn singleton_returns_its_value() {
        assert_eq!(majority_candidate(&[7]), 7);
    }

    #[test]
    fn returns_strict_majority() {
        assert_eq!(majority_candidate(&[1, 1, 2]), 1);
        assert_eq!(majority_candidate(&[1, 2, 2]), 2);
        assert_eq!(majority_candidate(&[2, 2, 1, 1, 1, 2, 2]), 2);
        assert_eq!(majority_candidate(&[1, 2, 1, 2, 1]), 1);
    }

    #[test]
    fn candidate_changes_after_cancellation() {
        assert_eq!(majority_candidate(&[1, 2, 3, 3, 3]), 3);
    }

    #[test]
    fn no_majority_follows_model_candidate_convention() {
        assert_eq!(majority_candidate(&[1, 2]), 1);
        assert_eq!(majority_candidate(&[1, 2, 1, 2]), 1);
        assert_eq!(majority_candidate(&[1, 2, 3, 4]), 3);
    }

    #[test]
    fn supports_full_u32_range() {
        assert_eq!(majority_candidate(&[u32::MAX, 0, u32::MAX]), u32::MAX);
        assert_eq!(majority_candidate(&[0, u32::MAX, 0]), 0);
    }

    #[test]
    fn supports_oracle_max_length() {
        let votes = vec![7; 4096];
        assert_eq!(majority_candidate(&votes), 7);
    }

}
