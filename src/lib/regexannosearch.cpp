#include "regexannosearch.h"
#include <limits>

using namespace annis;

RegexAnnoSearch::RegexAnnoSearch(const DB &db, const std::string& ns,
                                 const std::string& name, const std::string& valRegex)
  : db(db),
    validAnnotationsInitialized(false), valRegex(valRegex),
    compiledValRegex(valRegex),
    annoTemplate(Init::initAnnotation()),
    innerSearch(db, ns, name),
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
  innerSearch.reset();
  currentMatchValid = false;
}

RegexAnnoSearch::RegexAnnoSearch(const DB &db,
                                 const std::string& name, const std::string& valRegex)
  : db(db),
    validAnnotationsInitialized(false), valRegex(valRegex),
    compiledValRegex(valRegex),
    annoTemplate(Init::initAnnotation()),
    innerSearch(db, name),
    currentMatchValid(false)
{
  std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
  if(nameID.first)
  {
    annoTemplate.name = nameID.second;
  }
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
    while(innerSearch.hasNext())
    {
      Match candidate = innerSearch.next();
      if(RE2::FullMatch(db.strings.str(candidate.anno.val), compiledValRegex))
      {
        currentMatch = candidate;
        currentMatchValid = true;
        return;
      }
    }
  }
}



