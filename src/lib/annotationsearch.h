#ifndef ANNOTATIONSEARCH_H
#define ANNOTATIONSEARCH_H

#include "db.h"
#include "annotationiterator.h"

namespace annis
{

class AnnotationNameSearch : public AnnotationIterator
{
typedef stx::btree_multimap<std::uint32_t, Annotation>::const_iterator ItType;

public:
  AnnotationNameSearch(DB& db, std::string annoName);

  bool hasNext();
  const Annotation& next();

private:
  DB& db;
  std::string annoName;

  ItType it_lower;
  ItType it_upper;

};
} // end namespace annis
#endif // ANNOTATIONSEARCH_H
