#ifndef ANNOTATIONSEARCH_H
#define ANNOTATIONSEARCH_H

#include <annis/db.h>
#include <annis/iterators.h>
#include <annis/util/comparefunctions.h>

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
