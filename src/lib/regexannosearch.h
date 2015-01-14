#ifndef REGEXANNOSEARCH_H
#define REGEXANNOSEARCH_H

#include "annotationsearch.h"

#include <re2/re2.h>

namespace annis
{
class RegexAnnoSearch : public AnnotationSearch
{
public:
  RegexAnnoSearch(const DB& db, const std::string &name, const std::string &valRegex);
  RegexAnnoSearch(const DB& db, const std::string &ns, const std::string &name, const std::string &valRegex);

  virtual const std::set<Annotation, compAnno>& getValidAnnotations()
  {
    if(!validAnnotationsInitialized)
    {
      initValidAnnotations();
    }
    return validAnnotations;
  }

  virtual bool hasNext()
  {
    if(!currentMatchValid)
    {
      internalNextAnno();
    }
    return currentMatchValid;
  }

  virtual Match next();
  virtual void reset();

  virtual ~RegexAnnoSearch();
private:
    const DB& db;
    std::set<Annotation, compAnno> validAnnotations;
    bool validAnnotationsInitialized;
    std::string valRegex;
    RE2 compiledValRegex;
    Annotation annoTemplate;
    AnnotationNameSearch innerSearch;


    Match currentMatch;
    bool currentMatchValid;

    void initValidAnnotations();
    void internalNextAnno();

};
} // end namespace annis
#endif // REGEXANNOSEARCH_H
