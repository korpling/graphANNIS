#include "regexannosearch.h"
#include <limits>

using namespace annis;

RegexAnnoSearch::RegexAnnoSearch(const DB &db, const std::string& ns,
                                 const std::string& name, const std::string& valRegex)
  : db(db),
    validAnnotationsInitialized(false), valRegex(valRegex),
    compiledValRegex(valRegex),
    annoTemplate(Init::initAnnotation()),
    currentMatchValid(false)
{
  std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
  if(nameID.first)
  {
    annoTemplate.name = nameID.second;
  }
  std::pair<bool, std::uint32_t> namespaceID = db.strings.findID(ns);
  if(namespaceID.first)
  {
    annoTemplate.ns = namespaceID.second;
  }

  if(nameID.first && namespaceID.first)
  {
    itBegin = db.inverseNodeAnnotations.lower_bound({nameID.second, namespaceID.second, 0});
    it = itBegin;
    itEnd = db.inverseNodeAnnotations.upper_bound({nameID.second, namespaceID.second, uintmax});
  }
  else if(nameID.first)
  {
    itBegin = db.inverseNodeAnnotations.lower_bound({nameID.second, 0, 0});
    it = itBegin;
    itEnd = db.inverseNodeAnnotations.upper_bound({nameID.second, uintmax, uintmax});
  }
  else
  {
    itBegin = db.inverseNodeAnnotations.end();
    it = itBegin;
    itEnd = db.inverseNodeAnnotations.end();
  }

}


RegexAnnoSearch::RegexAnnoSearch(const DB &db,
                                 const std::string& name, const std::string& valRegex)
  : db(db),
    validAnnotationsInitialized(false), valRegex(valRegex),
    compiledValRegex(valRegex),
    annoTemplate(Init::initAnnotation()),
    currentMatchValid(false)
{
  std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
  if(nameID.first)
  {
    annoTemplate.name = nameID.second;

    itBegin = db.inverseNodeAnnotations.lower_bound({nameID.second, 0, 0});
    it = itBegin;
    itEnd = db.inverseNodeAnnotations.upper_bound({nameID.second, uintmax, uintmax});
  }
}

Match RegexAnnoSearch::next()
{
  if(!currentMatchValid)
  {
    internalNextAnno();
  }
  currentMatchValid = false;
  return currentMatch;
}

void RegexAnnoSearch::reset()
{
  it = itBegin;
  currentMatchValid = false;
}


RegexAnnoSearch::~RegexAnnoSearch()
{

}

void RegexAnnoSearch::initValidAnnotations()
{
  auto matchedStrings = db.strings.findRegex(valRegex);
  for(const auto& id : matchedStrings)
  {
    Annotation annoCopy = annoTemplate;
    annoCopy.val = id;
    validAnnotations.insert(annoCopy);
  }

  validAnnotationsInitialized = true;
}

void RegexAnnoSearch::internalNextAnno()
{
  currentMatchValid = false;
  if(compiledValRegex.ok())
  {
    while(it != itEnd)
    {
      Match candidate = {it->second, it->first};
      it++;

      if(RE2::FullMatch(db.strings.str(candidate.anno.val), compiledValRegex))
      {
        currentMatch = candidate;
        currentMatchValid = true;
        return;
      }
    }
  }
}



