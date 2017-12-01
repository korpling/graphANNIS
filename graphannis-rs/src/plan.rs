use std::rc::Rc;

pub struct ExecutionPlan {
    lhs : Rc<ExecutionPlan>,
    rhs : Rc<ExecutionPlan>,    
}