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
#include <random>
#include <re2/re2.h>

using namespace annis;

Query::Query(const DB &db, bool optimize)
  : db(db), optimize(optimize)
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

void Query::addOperator(std::shared_ptr<Operator> op, size_t idxLeft, size_t idxRight, bool forceNestedLoop)
{
  bestPlan.reset();

  OperatorEntry entry;
  entry.op = op;
  entry.forceNestedLoop = forceNestedLoop;
  entry.idxLeft = idxLeft;
  entry.idxRight = idxRight;
  
  operators.push_back(entry);
}

void Query::optimizeOperandOrder()
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
  const std::vector<OperatorEntry>& operators) 
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
      
      std::shared_ptr<ExecutionNode> joinExec = Plan::join(e.op, e.idxLeft, e.idxRight,
          execLeft, execRight, db, e.forceNestedLoop);
      node2exec[e.idxLeft] = joinExec;
      node2exec[e.idxRight] = joinExec;
      component2exec[joinExec->componentNr] = joinExec;
      
      
    }
  }
  
   // 3. check if there is only one component left (all nodes are connected)
  bool firstComponent = true;
  int firstComponentID;
  for(const auto& componentEntry : component2exec)
  {
    if(firstComponent)
    {
      firstComponent = false;
      firstComponentID = componentEntry.second->componentNr;
    }
    else
    {
      if(firstComponentID != componentEntry.second->componentNr)
      {
        std::cerr << "Nodes  are not completly connected, failing" << std::endl;
        return std::shared_ptr<Plan>();
      }
    }
  }
  
  return std::make_shared<Plan>(component2exec[firstComponentID]);
}



void Query::internalInit()
{
  if(bestPlan) {
    return;
  }
  
  if(optimize)
  {
    // use a constant seed to make the result deterministic
    std::mt19937 randGen(4711);
        
    ///////////////////////////////////////////////////////////
    // 1. make sure all smaller operand are on the left side //
    ///////////////////////////////////////////////////////////
    optimizeOperandOrder();
    
    if(operators.size() > 1)
    {
      ////////////////////////////////////
      // 2. optimize the order of joins //
      ////////////////////////////////////
      std::vector<OperatorEntry> optimizedOperators = operators;
      bestPlan = createPlan(nodes, optimizedOperators);
      double bestCost = bestPlan->getCost();

      // repeat until best plan is found
      const int maxUnsuccessfulTries = 10;
      int unsuccessful = 0;
      do
      {
        // randomly select two joins,        
        std::uniform_int_distribution<> dist(0, optimizedOperators.size()-1);
        int a, b;
        do
        {
          a = dist(randGen);
          b = dist(randGen);
        } while(a == b);
        
        // switch the order of the selected joins and check if the result has a smaller cost
        OperatorEntry tmpEntry = optimizedOperators[a];
        optimizedOperators[a] = optimizedOperators[b];
        optimizedOperators[b] = tmpEntry;
        
        auto altPlan = createPlan(nodes, optimizedOperators);
        double altCost = altPlan->getCost();

        if(altCost < bestCost)
        {
          bestPlan = altPlan;
          bestCost = altCost;
          unsuccessful = 0;
        }
        else
        {        
          unsuccessful++;
        }
      } while(unsuccessful < maxUnsuccessfulTries);
    } // end optimize join order
    else
    {
      bestPlan = createPlan(nodes, operators);
    }
  }
  else
  {
    // create unoptimized plan
    bestPlan = createPlan(nodes, operators);
  }
  
  currentResult.resize(nodes.size());
}


bool Query::next()
{
  if(!bestPlan)
  {
    internalInit();
  }
  
  if(bestPlan)
  {
    return bestPlan->executeStep(currentResult);
  }
  else
  {
    return false;
  }
}



