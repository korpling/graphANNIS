#pragma once

#include "annotationsearch.h"

#include <boost/container/flat_map.hpp>

namespace annis
{

class ExactAnnoValueSearch : public AnnotationSearch
{
  using ItType = boost::container::flat_multimap<Annotation, nodeid_t>::const_iterator;
  using Range = std::pair<ItType, ItType>;

public:

  /**
   * @brief Find annotations by name
   * @param db
   * @param annoName
   */
  ExactAnnoValueSearch(const DB &db, const std::string& annoNamspace, const std::string& annoName, const std::string& annoValue);
  ExactAnnoValueSearch(const DB &db, const std::string& annoName, const std::string& annoValue);

  virtual ~ExactAnnoValueSearch();


  virtual bool next(Match& result) override;
  virtual void reset();

  const std::unordered_set<Annotation>& getValidAnnotations()
  {
    if(!validAnnotationInitialized)
    {
      initializeValidAnnotations();
    }
    return validAnnotations;
  }
  
  std::int64_t guessMaxCount() const override;


private:
  const DB& db;

  std::list<Range> searchRanges;
  std::list<Range>::const_iterator currentRange;
  ItType it;

  bool validAnnotationInitialized;
  std::unordered_set<Annotation> validAnnotations;

  void initializeValidAnnotations();

};


} // end namespace annis

