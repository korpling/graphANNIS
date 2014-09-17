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
  AnnotationNameSearch(DB& db, std::string annoName);

  virtual bool hasNext()
  {
    return it != itEnd;
  }
  virtual Match next();

private:
  DB& db;
  std::string annoName;

  ItType it;
  ItType itEnd;

};
} // end namespace annis
#endif // ANNOTATIONSEARCH_H
