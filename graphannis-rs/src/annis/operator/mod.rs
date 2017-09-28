use annis::Match;

pub trait Operator {
    fn retrieve_matches() -> Box<Iterator<Item = Match>>;

    fn filter_match(lhs : &Match, rhs : &Match) -> bool;
}