#ifndef ANNOTATIONSEARCH_H
#define ANNOTATIONSEARCH_H

#include "db.h"
#include "iterators.h"

#include <set>

namespace annis
{

class AnnotationSearch : public CacheableAnnoIt
{
public:
  virtual const std::set<Annotation, compAnno>& getValidAnnotations() = 0;
  virtual ~AnnotationSearch() {};
};

class AnnotationNameSearch : public AnnotationSearch
{
  using ItType = stx::btree_multimap<Annotation, nodeid_t, compAnno>::const_iterator;

public:
  /**
   * @brief Find all annotations.
   * @param db
   */
  AnnotationNameSearch(DB& db);
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
  virtual Match current();
  virtual void reset();

  const std::set<Annotation, compAnno>& getValidAnnotations()
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
  std::set<Annotation, compAnno> validAnnotations;

  bool currentMatchValid;
  Match currentMatch;

  void initializeValidAnnotations();

};
} // end namespace annis
#endif // ANNOTATIONSEARCH_H
