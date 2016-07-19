#pragma once

#include "annotationsearch.h"
#include "exactannokeysearch.h"

#include <re2/re2.h>

namespace annis
{

  class RegexAnnoSearch : public AnnotationSearch
  {
    using AnnoItType = boost::container::flat_multimap<Annotation, nodeid_t>::const_iterator;
    using Range = std::pair<AnnoItType, AnnoItType>;

  public:
    RegexAnnoSearch(const DB& db, const std::string &name, const std::string &valRegex);
    RegexAnnoSearch(const DB& db, const std::string &ns, const std::string &name, const std::string &valRegex);

    virtual const std::unordered_set<Annotation>& getValidAnnotations()
    {
      if (!validAnnotationsInitialized)
      {
        initValidAnnotations();
      }
      return validAnnotations;
    }

    virtual const std::set<AnnotationKey>& getValidAnnotationKeys()
    {
      return validAnnotationKeys;
    }
    
    virtual bool next(Match& result) override;
    virtual void reset() override;

    std::int64_t guessMaxCount() const override;

    virtual ~RegexAnnoSearch();
  private:
    const DB& db;
    std::unordered_set<Annotation> validAnnotations;
    bool validAnnotationsInitialized;

    // always empty
    std::set<AnnotationKey> validAnnotationKeys;

    std::string valRegex;
    RE2 compiledValRegex;
    std::vector<Annotation> annoTemplates;

    std::list<Range> searchRanges;
    std::list<Range>::const_iterator currentRange;
    AnnoItType it;

  private:
    
    void initValidAnnotations();
    
  };
} // end namespace annis
