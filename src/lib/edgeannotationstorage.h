#ifndef EDGEANNOTATIONSTORAGE_H
#define EDGEANNOTATIONSTORAGE_H

#include "types.h"
#include <vector>
#include <stx/btree_multimap>

namespace annis
{

class EdgeAnnotationStorage
{
public:
  EdgeAnnotationStorage();
  virtual ~EdgeAnnotationStorage();

  virtual void addEdgeAnnotation(const Edge& edge, const Annotation& anno)
  {
    edgeAnnotations.insert2(edge, anno);
  }

  virtual void clear();

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const
  {
    typedef stx::btree_multimap<Edge, Annotation>::const_iterator ItType;

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

private:
  stx::btree_multimap<Edge, Annotation> edgeAnnotations;
};

} // end namespace annis

#endif // EDGEANNOTATIONSTORAGE_H
