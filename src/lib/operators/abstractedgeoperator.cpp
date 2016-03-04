#include <annis/operators/abstractedgeoperator.h>

#include <annis/wrapper.h>
#include <annis/util/comparefunctions.h>

#include <stx/btree_set>

using namespace annis;

AbstractEdgeOperator::AbstractEdgeOperator(
    ComponentType componentType,
    const DB &db, std::string ns, std::string name,
    unsigned int minDistance, unsigned int maxDistance)
  : componentType(componentType), db(db), ns(ns), name(name),
                  minDistance(minDistance), maxDistance(maxDistance),
                  anyAnno(Init::initAnnotation()), edgeAnno(anyAnno)
{
  initGraphStorage();
}

AbstractEdgeOperator::AbstractEdgeOperator(
    ComponentType componentType,
    const DB &db, std::string ns, std::string name, const Annotation &edgeAnno)
  : componentType(componentType), db(db), ns(ns), name(name),
    minDistance(1), maxDistance(1),
    anyAnno(Init::initAnnotation()), edgeAnno(edgeAnno)
{
  initGraphStorage();
}

std::unique_ptr<AnnoIt> AbstractEdgeOperator::retrieveMatches(const Match &lhs)
{
  std::unique_ptr<ListWrapper> w = std::make_unique<ListWrapper>();


  // add the rhs nodes of all of the edge storages
  if(gs.size() == 1)
  {
    std::unique_ptr<EdgeIterator> it = gs[0]->findConnected(lhs.node, minDistance, maxDistance);
    for(auto m = it->next(); m.first; m = it->next())
    {
      if(checkEdgeAnnotation(gs[0], lhs.node, m.second))
      {
        // directly add the matched node since when having only one component
        // no duplicates are possible
        w->addMatch(m.second);
      }
    }
  }
  else if(gs.size() > 1)
  {
    stx::btree_set<nodeid_t> uniqueResult;
    for(auto e : gs)
    {
      std::unique_ptr<EdgeIterator> it = e->findConnected(lhs.node, minDistance, maxDistance);
      for(auto m = it->next(); m.first; m = it->next())
      {
        if(checkEdgeAnnotation(e, lhs.node, m.second))
        {
          uniqueResult.insert(m.second);
        }
      }
    }
    for(const auto& n : uniqueResult)
    {
      w->addMatch(n);
    }
  }
  return w;
}

bool AbstractEdgeOperator::filter(const Match &lhs, const Match &rhs)
{
  // check if the two nodes are connected in *any* of the edge storages
  for(auto e : gs)
  {
    if(e->isConnected(Init::initEdge(lhs.node, rhs.node), minDistance, maxDistance))
    {
      if(checkEdgeAnnotation(e, lhs.node, rhs.node))
      {
        return true;
      }
    }
  }
  return false;
}


void AbstractEdgeOperator::initGraphStorage()
{
  if(ns == "")
  {
    gs = db.getGraphStorage(componentType, name);
  }
  else
  {
    // directly add the only known edge storage
    const ReadableGraphStorage* e = db.getGraphStorage(componentType, ns, name);
    if(e != nullptr)
    {
      gs.push_back(e);
    }
  }
}

bool AbstractEdgeOperator::checkEdgeAnnotation(const ReadableGraphStorage* e, nodeid_t source, nodeid_t target)
{
  if(edgeAnno == anyAnno)
  {
    return true;
  }
  else if(edgeAnno.val == 0)
  {
    // must be a valid value
    return false;
  }
  else
  {
    // check if the edge has the correct annotation first
    auto edgeAnnoList = e->getEdgeAnnotations(Init::initEdge(source, target));
    for(const auto& anno : edgeAnnoList)
    {
      if(checkAnnotationEqual(edgeAnno, anno))
      {
        return true;
      }
    } // end for each annotation of candidate edge
  }
  return false;
}

double AbstractEdgeOperator::selectivity() 
{
  if(gs.size() == 0)
  {
    // will not find anything
    return 0.0;
  }
  
  double sumSel = 0.0;
  
  for(const ReadableGraphStorage* g: gs)
  {
    const auto& stat = g->getStatistics();
    if(stat.cyclic)
    {
      // can get all other nodes
      return 1.0;
    }
    
    // get number of nodes reachable from min to max distance
    std::uint32_t maxPathLength = std::min(maxDistance, stat.maxDepth);
    std::uint32_t minPathLength = std::max(0, (int) minDistance-1);
    
    std::uint32_t reachableMax = std::ceil(stat.avgFanOut * (double) maxPathLength);
    std::uint32_t reachableMin = std::ceil(stat.avgFanOut * (double) minPathLength);
    
    std::uint32_t reachable =  reachableMax - reachableMin;
    
    sumSel += ((double) reachable ) / ((double) stat.nodes);
  }
  
  // return average selectivity
  return sumSel / (double) gs.size();
}


std::string AbstractEdgeOperator::description() 
{
  std::string result;
  if(minDistance == 1 && maxDistance == 1)
  {
    result =  operatorString() + name;
  }
  else if(minDistance == 1 && maxDistance == std::numeric_limits<unsigned int>::max())
  {
    result = operatorString() + name + " *";
  }
  else if(minDistance == maxDistance)
  {
    result = operatorString() + name + "," + std::to_string(minDistance);
  }
  else
  {
    result = operatorString() + name + "," + std::to_string(minDistance) + "," + std::to_string(maxDistance);
  }
  
  if(!(edgeAnno == anyAnno))
  {
    if(edgeAnno.name != 0 && edgeAnno.val != 0)
    {
      result += "[" + db.strings.str(edgeAnno.name) + "=\"" + db.strings.str(edgeAnno.val) + "\"]";
    }
    else
    {
      result += "[invalid anno]";
    }
  }
  
  return result;
}


AbstractEdgeOperator::~AbstractEdgeOperator()
{

}

