/*
Link culler

We want to work on message data, so that we're dealing with smaller data sets when we score / tag links.

Inputs:
json of messages

Mutators:
Apply exclusion rules

Outputs:
json of messages

*/
pub trait Cull {
    fn cull(&mut self, cullstring: &str);
    fn cull_list(&mut self, culls: Vec<&str>);
    fn keep(&mut self, keepstring: &str);
    fn keep_list(&mut self, keeps: Vec<&str>);
    fn dedupe(&mut self);
}
