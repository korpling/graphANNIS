#ifndef PREPOSTORDERSTORAGE_H
#define PREPOSTORDERSTORAGE_H

#include "fallbackedgedb.h"

#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stack>

namespace annis
{

struct Node
{
  nodeid_t id;
  /**
   * @brief id of the root node of the subcomponent
   */
  nodeid_t root;
};

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

template<>
struct less<annis::Node>
{
  bool operator()(const struct annis::Node &a, const struct annis::Node &b) const
  {
    // compare by id
    if(a.id < b.id) {return true;} else if(a.id > b.id) {return false;}

    // compare by root node
    if(a.root < b.root) {return true;} else if(a.root > b.root) {return false;}

    // they are equal
    return false;
  }
};

} // end namespace std

namespace annis
{

class PrePostOrderStorage : public FallbackEdgeDB
{  
friend class PrePostIterator;

public:
  PrePostOrderStorage(StringStorage& strings, const Component& component);
  virtual ~PrePostOrderStorage();

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual void calculateIndex();

  virtual bool isConnected(const Edge& edge, unsigned int minDistance = 1, unsigned int maxDistance = 1) const;
  virtual int distance(const Edge &edge) const;
  virtual std::unique_ptr<EdgeIterator> findConnected(nodeid_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const;

private:
  stx::btree_map<Node, PrePost> node2order;
  stx::btree_map<uint32_t, nodeid_t> order2node;


  void enterNode(uint32_t& currentOrder, nodeid_t nodeID, nodeid_t rootNode, int32_t level, std::stack<nodeid_t> &nodeStack);
  void exitNode(uint32_t &currentOrder, std::stack<nodeid_t> &nodeStack, nodeid_t rootNode);

};

class PrePostIterator : public EdgeIterator
{
  using OrderIt = stx::btree_map<uint32_t, nodeid_t>::const_iterator;
public:

  PrePostIterator(const PrePostOrderStorage& storage, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance);

  virtual std::pair<bool, nodeid_t> next();

  virtual void reset();

  virtual ~PrePostIterator();
private:

  const PrePostOrderStorage& storage;
  unsigned int minDistance;
  unsigned int maxDistance;
  std::uint32_t startNode;

  std::stack<std::pair<OrderIt, OrderIt> > ranges;
  OrderIt currentNode;

};

} // end namespace annis


#endif // PREPOSTORDERSTORAGE_H
