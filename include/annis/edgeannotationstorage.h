#pragma once

#include "types.h"
#include <vector>
#include <google/btree_map.h>

namespace annis
{

class EdgeAnnotationStorage
{
public:

  template<typename Key, typename Value>
  using multimap_t = btree::btree_multimap<Key, Value>;

  EdgeAnnotationStorage();
  virtual ~EdgeAnnotationStorage();

  virtual void addEdgeAnnotation(const Edge& edge, const Annotation& anno)
  {
    edgeAnnotations.insert({edge, anno});
  }

  virtual void clear();

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const
  {
    typedef multimap_t<Edge, Annotation>::const_iterator ItType;

    std::vector<Annotation> result;

    std::pair<ItType, ItType> range =
        edgeAnnotations.equal_range(edge);

    for(ItType it=range.first; it != range.second; ++it)
    {
      result.push_back(it->second);
    }

    return result;
  }

  virtual std::uint32_t numberOfEdgeAnnotations() const
  {
    return edgeAnnotations.size();
  }

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  size_t estimateMemorySize();

private:
  multimap_t<Edge, Annotation> edgeAnnotations;
};

} // end namespace annis
