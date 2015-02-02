#ifndef PREPOSTORDERSTORAGE_H
#define PREPOSTORDERSTORAGE_H

#include "fallbackedgedb.h"
#include "edgeannotationstorage.h"

#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stack>
#include <list>


namespace annis
{
struct PrePost
{
  uint32_t pre;
  uint32_t post;
  int32_t level;
};
} // end namespace annis

namespace std
{
template<>
struct less<annis::PrePost>
{
  bool operator()(const struct annis::PrePost &a, const struct annis::PrePost &b) const
  {
    // compare by pre-order
    if(a.pre < b.pre) {return true;} else if(a.pre > b.pre) {return false;}

    // compare by post-order
    if(a.post < b.post) {return true;} else if(a.post > b.post) {return false;}

    // compare by level
    if(a.level < b.level) {return true;} else if(a.level > b.level) {return false;}

    // they are equal
    return false;
  }
};

} // end namespace std

namespace annis
{

struct SearchRange
{
  stx::btree_map<PrePost, nodeid_t>::const_iterator lower;
  stx::btree_map<PrePost, nodeid_t>::const_iterator upper;
  uint32_t maximumPost;
  int32_t startLevel;
};


struct NodeStackEntry
{
  nodeid_t id;
  PrePost order;
};

class PrePostOrderStorage : public ReadableGraphStorage
{  
friend class PrePostIterator;
using NStack = std::stack<NodeStackEntry, std::list<NodeStackEntry> >;

public:
  PrePostOrderStorage(StringStorage& strings, const Component& component);
  virtual ~PrePostOrderStorage();

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual void copy(const DB& db, const ReadableGraphStorage& orig);

  virtual void clear();

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const
  {
    return edgeAnno.getEdgeAnnotations(edge);
  }

  virtual bool isConnected(const Edge& edge, unsigned int minDistance = 1, unsigned int maxDistance = 1) const;
  virtual int distance(const Edge &edge) const;
  virtual std::unique_ptr<EdgeIterator> findConnected(nodeid_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const;

  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const;
  virtual std::vector<nodeid_t> getIncomingEdges(nodeid_t node) const;

  virtual std::uint32_t numberOfEdges() const
  {
    return order2node.size();
  }
  virtual std::uint32_t numberOfEdgeAnnotations() const
  {
    return edgeAnno.numberOfEdgeAnnotations();
  }

private:
  stx::btree_multimap<nodeid_t, PrePost> node2order;
  stx::btree_map<PrePost, nodeid_t> order2node;
  EdgeAnnotationStorage edgeAnno;

  void enterNode(uint32_t& currentOrder, nodeid_t nodeID, nodeid_t rootNode, int32_t level, NStack &nodeStack);
  void exitNode(uint32_t &currentOrder, NStack &nodeStack);

};

class PrePostIterator : public EdgeIterator
{
  using OrderIt = stx::btree_map<PrePost, nodeid_t>::const_iterator;
public:

  PrePostIterator(const PrePostOrderStorage& storage,
                  const nodeid_t& startNode,
                  const unsigned int& minDistance,
                  const unsigned int& maxDistance);

  virtual std::pair<bool, nodeid_t> next();

  virtual void reset();

  virtual ~PrePostIterator();
private:

  const PrePostOrderStorage& storage;
  const unsigned int minDistance;
  const unsigned int maxDistance;
  const nodeid_t startNode;

  std::stack<SearchRange, std::list<SearchRange> > ranges;
  OrderIt currentNode;

  stx::btree_set<nodeid_t> visited;

  void init();

};

} // end namespace annis


#endif // PREPOSTORDERSTORAGE_H
