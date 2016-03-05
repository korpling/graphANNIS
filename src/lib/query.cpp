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
  entry.originalOrder = operators.size();
  
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

std::shared_ptr<const Plan> Query::getBestPlan() 
{
  if(!bestPlan)
  {
    internalInit();
  }
  return bestPlan;
}


std::shared_ptr<Plan> Query::createPlan(const std::vector<std::shared_ptr<AnnoIt> >& nodes, 
  const std::vector<OperatorEntry>& operators) 
{
  std::map<nodeid_t, int> node2component;
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
    node2component[i] = i;
    component2exec[i] = baseNode;
    i++;
  }
  const size_t numOfNodes = i;
  
  // 2. add the operators which produce the results
  for(auto& e : operators)
  {
    if(e.idxLeft < numOfNodes && e.idxRight < numOfNodes)
    {
      
      int componentLeft = node2component[e.idxLeft];
      int componentRight = node2component[e.idxRight];
      
      std::shared_ptr<ExecutionNode> execLeft = component2exec[componentLeft];
      std::shared_ptr<ExecutionNode> execRight = component2exec[componentRight];
      
      std::shared_ptr<ExecutionNode> joinExec = Plan::join(e.op, e.idxLeft, e.idxRight,
          execLeft, execRight, db, e.forceNestedLoop);
      updateComponentForNodes(node2component, componentLeft, joinExec->componentNr);
      updateComponentForNodes(node2component, componentRight, joinExec->componentNr);
      component2exec[joinExec->componentNr] = joinExec;      
      
    }
  }
  
   // 3. check if there is only one component left (all nodes are connected)
  bool firstComponent = true;
  int firstComponentID;
  for(const auto& e : node2component)
  {
    if(firstComponent)
    {
      firstComponent = false;
      firstComponentID = e.second;
    }
    else
    {
      if(firstComponentID != e.second)
      {
        std::cerr << "Nodes  are not completly connected, failing" << std::endl;
        return std::shared_ptr<Plan>();
      }
    }
  }
  
  return std::make_shared<Plan>(component2exec[firstComponentID]);
}

void Query::updateComponentForNodes(std::map<nodeid_t, int>& node2component, int from, int to)
{
  if(from == to)
  {
    // nothing todo
    return;
  }

  std::vector<int> nodeIDs2update;
  for(const auto e : node2component)
  {
    if(e.second == from)
    {
      nodeIDs2update.push_back(e.first);
    }
  }
  // set the component id for each node of the other component
  for(auto nodeID : nodeIDs2update)
  {
    node2component[nodeID] = to;
  }
}



void Query::internalInit()
{
  if(bestPlan) {
    return;
  }
  
  if(optimize)
  {
    ///////////////////////////////////////////////////////////
    // 1. make sure all smaller operand are on the left side //
    ///////////////////////////////////////////////////////////
    optimizeOperandOrder();
    
    if(operators.size() > 1)
    {
      ////////////////////////////////////
      // 2. optimize the order of joins //
      ////////////////////////////////////
      if(operators.size() <= 7)
      {
        optimizeJoinOrderAllPermutations();
      }
      else
      {
        optimizeJoinOrderRandom();
      }
      
    } // end optimize join order
    else
    {
      bestPlan = createPlan(nodes, operators);
      // still get the cost so the estimates are calculated
      bestPlan->getCost();
    }
  }
  else
  {
    // create unoptimized plan
    bestPlan = createPlan(nodes, operators);
  }
  
  currentResult.resize(nodes.size());
}

void Query::optimizeJoinOrderRandom() 
{
  // use a constant seed to make the result deterministic
  std::mt19937 randGen(4711);
    
  std::vector<OperatorEntry> optimizedOperators = operators;
  bestPlan = createPlan(nodes, optimizedOperators);
  double bestCost = bestPlan->getCost();

//  std::cout << "orig plan:" << std::endl;
//  std::cout << operatorOrderDebugString(optimizedOperators) << std::endl;
//  std::cout << bestPlan->debugString() << std::endl;
//  std::cout << "-----------------------" << std::endl;

  // repeat until best plan is found
  const int maxUnsuccessfulTries = 20*operators.size();
  int unsuccessful = 0;
  do
  {

    std::vector<OperatorEntry> tmpOperators = optimizedOperators;
    // randomly select two joins,        
    std::uniform_int_distribution<> dist(0, tmpOperators.size()-1);
    int a, b;
    do
    {
      a = dist(randGen);
      b = dist(randGen);
    } while(a == b);


    // switch the order of the selected joins and check if the result has a smaller cost
    OperatorEntry tmpEntry = tmpOperators[a];
    tmpOperators[a] = tmpOperators[b];
    tmpOperators[b] = tmpEntry;

    auto altPlan = createPlan(nodes, tmpOperators);
    double altCost = altPlan->getCost();

//    std::cout << "................................" << std::endl;
//    std::cout << "try to switch op " << a << " with op " << b << std::endl;
//    std::cout << operatorOrderDebugString(tmpOperators) << std::endl;
//    std::cout << altPlan->debugString() << std::endl;
//    std::cout << "................................" << std::endl;

    if(altCost < bestCost)
    {
      bestPlan = altPlan;
      optimizedOperators = tmpOperators;

//      std::cout << "================================" << std::endl;
//      std::cout << "new plan:" << std::endl;
//      std::cout << operatorOrderDebugString(optimizedOperators) << std::endl;
//      std::cout << bestPlan->debugString() << std::endl;
//      std::cout << "================================" << std::endl;

      bestCost = altCost;
      unsuccessful = 0;
    }
    else
    {        
      unsuccessful++;
    }
  } while(unsuccessful < maxUnsuccessfulTries);
}

void Query::optimizeJoinOrderAllPermutations() 
{
  // make sure the first permutation is the sorted one
  std::vector<OperatorEntry> testOrder = operators;
  std::sort(testOrder.begin(), testOrder.end(), compare_opentry_origorder);
  
  bestPlan = createPlan(nodes, testOrder);

//  std::cout << operatorOrderDebugString(testOrder) << std::endl;
//  std::cout << bestPlan->debugString() << std::endl;
//  std::cout << "-------------------------------" << std::endl;
  
  while(std::next_permutation(testOrder.begin(), testOrder.end(), compare_opentry_origorder))
  {
    std::shared_ptr<Plan> testPlan = createPlan(nodes, testOrder);
//    testPlan->getCost();
//    std::cout << operatorOrderDebugString(testOrder) << std::endl;
//    std::cout << testPlan->debugString() << std::endl;
    
    if(testPlan->getCost() < bestPlan->getCost())
    {
      bestPlan = testPlan;
      
//      std::cout << "!!!new best join order!!! " << std::endl;
    }
//    std::cout << "-------------------------------" << std::endl;
  }
}



std::string Query::operatorOrderDebugString(const std::vector<OperatorEntry>& ops) 
{
  std::string result = "";
  for(auto it=ops.begin(); it != ops.end(); it++)
  {
    if(it != ops.begin())
    {
      result += " | ";
    }
    if(it->op)
    {
      result += "#" + std::to_string(it->idxLeft+1) + " " +
        it->op->description()
        + " #" + std::to_string(it->idxRight+1);
    }
    else
    {
      result += "<empty>";
    }
  }
  
  return result;
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



