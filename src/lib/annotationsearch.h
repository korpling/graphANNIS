#ifndef ANNOTATIONSEARCH_H
#define ANNOTATIONSEARCH_H

#include "db.h"
#include "annotationiterator.h"

namespace annis
{

class AnnotationNameSearch : public AnnotationIterator
{
typedef stx::btree_multimap<Annotation, std::uint32_t, compAnno>::const_iterator ItType;

public:
  AnnotationNameSearch(DB& db, const std::string& annoName);
  AnnotationNameSearch(DB& db, const std::string& annoNamspace, const std::string& annoName, const std::string& annoValue);

  virtual bool hasNext()
  {
    return it != itEnd;
  }
  virtual Match next();

private:
  DB& db;

  ItType it;
  ItType itEnd;

};
} // end namespace annis
#endif // ANNOTATIONSEARCH_H
