#ifndef ANNOTATIONSEARCH_H
#define ANNOTATIONSEARCH_H

#include "db.h"
#include "iterators.h"
#include "comparefunctions.h"

#include <set>
#include <unordered_set>

namespace annis
{

class AnnotationSearch : public AnnoIt
{
public:
  virtual const std::unordered_set<Annotation>& getValidAnnotations() = 0;

  virtual ~AnnotationSearch() {};
};

class AnnotationKeySearch : public AnnoIt
{
public:
  virtual const std::set<AnnotationKey>& getValidAnnotationKeys() = 0;

  virtual ~AnnotationKeySearch() {};
};

} // end namespace annis
#endif // ANNOTATIONSEARCH_H
