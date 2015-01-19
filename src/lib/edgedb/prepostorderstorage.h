#ifndef PREPOSTORDERSTORAGE_H
#define PREPOSTORDERSTORAGE_H

#include "fallbackedgedb.h"

#include <stx/btree_map>

namespace annis
{

struct PrePost
{
  uint32_t pre;
  uint32_t post;
};


class PrePostOrderStorage : public FallbackEdgeDB
{
public:
  PrePostOrderStorage(StringStorage& strings, const Component& component);
  virtual ~PrePostOrderStorage();

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual void calculateIndex();

private:
  stx::btree_map<nodeid_t, PrePost> node2order;
  stx::btree_map<uint32_t, uint32_t> order2node;

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

    // they are equal
    return false;
  }
};

} // end namespace std

#endif // PREPOSTORDERSTORAGE_H
