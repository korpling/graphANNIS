#include <annis/query.h>
#include <annis/join/nestedloop.h>
#include <annis/join/seed.h>
#include <annis/filter.h>
#include <annis/operators/operator.h>
#include <annis/db.h>
#include <annis/iterators.h>
#include <annis/annosearch/annotationsearch.h>
#include <annis/wrapper.h>

#include <vector>
#include <re2/re2.h>

using namespace annis;

Query::Query(const DB &db)
  : db(db)
{
}

Query::~Query() {
  
}

size_t annis::Query::addNode(std::shared_ptr<AnnotationSearch> n, bool wrapAnyNodeAnno)
{
  bestPlan.reset();

  size_t idx = nodes.size();
  
  if(wrapAnyNodeAnno)
  {
    Annotation constAnno = {db.getNodeNameStringID(), db.getNamespaceStringID(), 0};
    nodes.push_back(std::make_shared<ConstAnnoWrapper>(constAnno, n));
  }
  else
  {
    nodes.push_back(n);
  }
  return idx;
}

size_t annis::Query::addNode(std::shared_ptr<AnnotationKeySearch> n, bool wrapAnyNodeAnno)
{
  bestPlan.reset();

  size_t idx = nodes.size();
  if(wrapAnyNodeAnno)
  {
    Annotation constAnno = {db.getNodeNameStringID(), db.getNamespaceStringID(), 0};
    nodes.push_back(std::make_shared<ConstAnnoWrapper>(constAnno, n));
  }
  else
  {
    nodes.push_back(n);
  }
  return idx;
}

void Query::addOperator(std::shared_ptr<Operator> op, size_t idxLeft, size_t idxRight, bool useNestedLoop)
{
  bestPlan.reset();

  OperatorEntry entry;
  entry.op = op;
  entry.useNestedLoop = useNestedLoop;
  entry.idxLeft = idxLeft;
  entry.idxRight = idxRight;
  
  operators.push_back(entry);
}

void Query::optimize()
{
  if(!bestPlan && db.nodeAnnos.hasStatistics())
  {
    // for each commutative operator check if is better to switch the operands
    for(auto& e : operators)
    {
      if(e.op && e.op->isCommutative() && e.idxLeft < nodes.size() && e.idxRight < nodes.size())
      {
        std::shared_ptr<EstimatedSearch> lhs = 
          std::dynamic_pointer_cast<EstimatedSearch>(nodes[e.idxLeft]);
        std::shared_ptr<EstimatedSearch> rhs = 
          std::dynamic_pointer_cast<EstimatedSearch>(nodes[e.idxRight]);
        
        if(lhs && rhs)
        {
          std::int64_t estimateLHS = lhs->guessMaxCount();
          std::int64_t estimateRHS = rhs->guessMaxCount();
          
          if(estimateLHS >= 0 && estimateRHS >= 0 && estimateLHS > estimateRHS)
          {
            // the left one is larger, so switch both operands
            size_t oldLeft = e.idxLeft;
            e.idxLeft = e.idxRight;
            e.idxRight = oldLeft;
          }

        }
      }
    }
    
    // TODO: optimize join order
  }
}

std::shared_ptr<Plan> Query::createPlan(const std::vector<std::shared_ptr<AnnoIt> >& nodes, 
  const std::list<OperatorEntry>& operators, const DB& db) 
{
  std::map<int, std::shared_ptr<ExecutionNode>> node2exec;
  std::map<int, std::shared_ptr<ExecutionNode>> component2exec;
  
  // 1. add all nodes
  int i=0;
  for(auto& n : nodes)
  {
    std::shared_ptr<ExecutionNode> baseNode = std::make_shared<ExecutionNode>();
    baseNode->type = ExecutionNodeType::base;
    baseNode->join = n;
    baseNode->nodePos[i] = 0;
    baseNode->componentNr = i;
    node2exec[i] = baseNode;
    component2exec[i] = baseNode;
    i++;
  }
  const size_t numOfNodes = i;
  
  // 2. add the operators which produce the results
  for(auto& e : operators)
  {
    if(e.idxLeft < numOfNodes && e.idxRight < numOfNodes)
    {
      
      std::shared_ptr<ExecutionNode> execLeft = node2exec[e.idxLeft];
      std::shared_ptr<ExecutionNode> execRight = node2exec[e.idxRight];
      
      if(execLeft->componentNr == execRight->componentNr)
      {        
        // the join is already fully completed inside a component, only filter
        std::shared_ptr<ExecutionNode> joinExec = Plan::join(e.op, e.idxLeft, e.idxRight,
          *execLeft, *execRight, db, ExecutionNodeType::filter);
        // replace the old top-level exec node with the new one
        node2exec[e.idxLeft] = joinExec;
        node2exec[e.idxRight] = joinExec;
        component2exec[execLeft->componentNr] = joinExec;
      }
      else
      {
        // this joins two components which each other
        component2exec.erase(execRight->componentNr);
        execRight->componentNr = execLeft->componentNr;
        
        ExecutionNodeType t = ExecutionNodeType::nested_loop;
        // if the right side is not another join we can use a seed join
        if(execRight->type == ExecutionNodeType::base)
        {
          t = ExecutionNodeType::seed;
        }
        std::shared_ptr<ExecutionNode> joinExec = Plan::join(e.op, e.idxLeft, e.idxRight,
          *execLeft, *execRight, db, t);
        // replace the old top-level exec node with the new one
        node2exec[e.idxLeft] = joinExec;
        node2exec[e.idxRight] = joinExec;
        component2exec[execLeft->componentNr] = joinExec;
      }
    }
  }
  
   // 3. check if there is only one component left (all nodes are connected)
  if(component2exec.size() == 1)
  {
    return std::make_shared<Plan>(component2exec.begin()->second);
  }
  else
  {
     std::cerr << "Nodes " << " are not completly connected, failing" << std::endl;
        return std::shared_ptr<Plan>();
  }
}



void Query::internalInit()
{
  if(bestPlan) {
    return;
  }

  bestPlan = createPlan(nodes, operators, db);
  currentResult.resize(nodes.size());
}


bool Query::next()
{
  if(!bestPlan)
  {
    internalInit();
  }
  
  return bestPlan->executeStep(currentResult);
}



