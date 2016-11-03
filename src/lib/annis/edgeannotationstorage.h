#pragma once

#include "types.h"
#include <vector>
#include <google/btree_map.h>

#include <annis/serializers.h>

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

  virtual void deleteEdgeAnnotation(const Edge& edge, const AnnotationKey& anno)
  {
    auto range = edgeAnnotations.equal_range(edge);
    for(auto it = range.first; it != range.second; it++)
    {
       if(it->second.ns == anno.ns && it->second.name == anno.name)
       {
          edgeAnnotations.erase(it);
       }
    }
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

  virtual size_t numberOfEdgeAnnotations() const
  {
    return edgeAnnotations.size();
  }

  template<class Archive>
  void serialize(Archive & archive)
  {
    archive(edgeAnnotations);
  }

  size_t estimateMemorySize();

private:
  multimap_t<Edge, Annotation> edgeAnnotations;
};

} // end namespace annis
