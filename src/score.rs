/// score is responsible for calculating the scores of the similarity between
/// the query and the choice.
///
/// It is modeled after https://github.com/felipesere/icepick.git

use std::cmp::max;
use std::cell::RefCell;
const BONUS_ADJACENCY: i32 = 5;
const BONUS_SEPARATOR: i32 = 10;
const BONUS_CAMEL: i32 = 10;
const PENALTY_LEADING: i32 = -3; // penalty applied for every letter before the first match
const PENALTY_MAX_LEADING: i32 = -9; // maxing penalty for leading letters
const PENALTY_UNMATCHED: i32 = -1;

// judge how many scores the current index should get
fn fuzzy_score(string: &Vec<char>, index: usize, is_first: bool) -> i32 {
    let mut score = 0;
    if index == 0 {
        return BONUS_SEPARATOR;
    }

    let prev = string[index-1];
    let cur = string[index];

    // apply bonus for matches after a separator
    if prev == ' ' || prev == '_' || prev == '-' || prev == '/' || prev == '\\' {
        score += BONUS_SEPARATOR;
    }

    // apply bonus for camelCases
    if prev.is_lowercase() && cur.is_uppercase() {
        score += BONUS_CAMEL;
    }

    if is_first {
        score += max((index as i32) * PENALTY_LEADING, PENALTY_MAX_LEADING);
    }

    score
}

pub fn fuzzy_match(choice: &str, pattern: &str) -> Option<(i32, Vec<usize>)>{
    if pattern.len() == 0 {
        return Some((0, Vec::new()));
    }

    let choice_chars: Vec<char> = choice.chars().collect();
    let choice_lower = choice.to_lowercase();
    let pattern_chars: Vec<char> = pattern.to_lowercase().chars().collect();

    let mut scores = vec![];
    let mut picked = vec![];

    let mut prev_matched_idx = -1; // to ensure that the pushed char are able to match the pattern
    for pattern_idx in 0..pattern_chars.len() {
        let pattern_char = pattern_chars[pattern_idx];
        let vec_cell = RefCell::new(vec![]);
        {
            let mut vec = vec_cell.borrow_mut();
            for (idx, ch) in choice_lower.chars().enumerate() {
                if ch == pattern_char && (idx as i32) > prev_matched_idx {
                    vec.push((idx, fuzzy_score(&choice_chars, idx, pattern_idx == 0), 0)); // (char_idx, score, vec_idx back_ref)
                }
            }

            if vec.len() <= 0 {
                // not matched
                return None;
            }
            prev_matched_idx = vec[0].0 as i32;
        }
        scores.push(vec_cell);
    }

    for pattern_idx in 0..pattern.len()-1 {
        let cur_row = scores[pattern_idx].borrow();
        let mut next_row = scores[pattern_idx+1].borrow_mut();

        for idx in 0..next_row.len() {
            let (next_char_idx, next_score, _) = next_row[idx];
//(back_ref, &score)
            let (back_ref, score) = cur_row.iter()
                .take_while(|&&(idx, _, _)| idx < next_char_idx)
                .map(|&(char_idx, score, _)| {
                    let adjacent_num = next_char_idx - char_idx - 1;
                    score + next_score + if adjacent_num == 0 {BONUS_ADJACENCY} else {PENALTY_UNMATCHED * adjacent_num as i32}
                })
                .enumerate()
                .max_by_key(|&(_, x)| x)
                .unwrap();

            next_row[idx] = (next_char_idx, score, back_ref);
        }
    }

    let (mut next_col, &(_, score, _)) = scores[pattern.len()-1].borrow().iter().enumerate().max_by_key(|&(_, &x)| x.1).unwrap();
    let mut pattern_idx = pattern.len() as i32 - 1;
    while pattern_idx >= 0 {
        let (idx, _, next) = scores[pattern_idx as usize].borrow()[next_col];
        next_col = next;
        picked.push(idx);
        pattern_idx -= 1;
    }
    picked.reverse();
    Some((score, picked))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compute_match_length() {
        let choice_1 = "I am a 中国人.";
        let query_1 = "a人";
        assert_eq!(super::compute_match_length(&choice_1, &query_1), Some((2, 8)));

        let choice_2 = "Choice did not matter";
        let query_2 = "";
        assert_eq!(super::compute_match_length(&choice_2, &query_2), Some((0, 0)));

        let choice_3 = "abcdefg";
        let query_3 = "hi";
        assert_eq!(super::compute_match_length(&choice_3, &query_3), None);

        let choice_4 = "Partial match did not count";
        let query_4 = "PP";
        assert_eq!(compute_match_length(&choice_4, &query_4), None);
    }

    #[test]
    fn teset_fuzzy_match() {
        // the score in this test doesn't actually matter, but the index matters.
        let choice_1 = "1111121";
        let query_1 = "21";
        assert_eq!(fuzzy_match(&choice_1, &query_1), Some((-4, vec![5,6])));

        let choice_2 = "Ca";
        let query_2 = "ac";
        assert_eq!(fuzzy_match(&choice_2, &query_2), None);

        let choice_3 = ".";
        let query_3 = "s";
        assert_eq!(fuzzy_match(&choice_3, &query_3), None);

        let choice_4 = "AaBbCc";
        let query_4 = "abc";
        assert_eq!(fuzzy_match(&choice_4, &query_4), Some((28, vec![0,2,4])));
    }
}