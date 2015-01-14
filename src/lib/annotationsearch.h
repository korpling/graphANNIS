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

class AnnotationNameSearch : public AnnotationSearch
{
  using ItType = stx::btree_multimap<Annotation, nodeid_t>::const_iterator;

public:
  /**
   * @brief Find all annotations.
   * @param db
   */
  AnnotationNameSearch(const DB& db);
  /**
   * @brief Find annotations by name
   * @param db
   * @param annoName
   */
  AnnotationNameSearch(const DB& db, const std::string& annoName);
  AnnotationNameSearch(const DB& db, const std::string& annoNamspace, const std::string& annoName);
  AnnotationNameSearch(const DB &db, const std::string& annoNamspace, const std::string& annoName, const std::string& annoValue);

  virtual ~AnnotationNameSearch();

  virtual bool hasNext()
  {
    return it != db.inverseNodeAnnotations.end() && it != itEnd;
  }

  virtual Match next();
  virtual void reset();

  const std::unordered_set<Annotation>& getValidAnnotations()
  {
    if(!validAnnotationInitialized)
    {
      initializeValidAnnotations();
    }
    return validAnnotations;
  }

private:
  const DB& db;

  ItType it;
  ItType itBegin;
  ItType itEnd;

  bool validAnnotationInitialized;
  std::unordered_set<Annotation> validAnnotations;

  bool currentMatchValid;
  Match currentMatch;

  void initializeValidAnnotations();

};
} // end namespace annis
#endif // ANNOTATIONSEARCH_H
