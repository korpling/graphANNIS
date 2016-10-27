#pragma once

#include <annis/db.h>
#include <annis/iterators.h>
#include <annis/util/comparefunctions.h>

#include <set>
#include <unordered_set>

namespace annis
{

class EstimatedSearch : public AnnoIt
{
public:
  virtual std::int64_t guessMaxCount() const {return -1;}
};

class AnnotationSearch : public EstimatedSearch
{
public:
  virtual const std::unordered_set<Annotation>& getValidAnnotations() = 0;
  
  virtual ~AnnotationSearch() {}
};

class AnnotationKeySearch : public EstimatedSearch
{
public:
  virtual const std::set<AnnotationKey>& getValidAnnotationKeys() = 0;
  
  virtual ~AnnotationKeySearch() {}
};

} // end namespace annis
