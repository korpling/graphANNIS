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
    auto lower = db.inverseNodeAnnotations.lower_bound({nameID.second, namespaceID.second, 0});
    auto upper = db.inverseNodeAnnotations.lower_bound({nameID.second, namespaceID.second, uintmax});
    searchRanges.push_back(Range(lower, upper));
  }
  currentRange = searchRanges.begin();
  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
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
  if(compiledValRegex.ok())
  {
    // get the size of the last element so we know how much our prefix needs to be
    size_t prefixSize = 10;
    if(!db.inverseNodeAnnotations.empty())
    {
      const Annotation lastAnno = db.inverseNodeAnnotations.rbegin()->first;
      size_t lastAnnoSize = db.strings.str(lastAnno.val).size()+1;
      if(lastAnnoSize > prefixSize)
      {
        prefixSize = lastAnnoSize;
      }
    }

    std::string minPrefix;
    std::string maxPrefix;
    compiledValRegex.PossibleMatchRange(&minPrefix, &maxPrefix, prefixSize);
    uint32_t lowerVal = db.strings.lower_bound(minPrefix);
    uint32_t upperVal = db.strings.upper_bound(maxPrefix);

    std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
    if(nameID.first)
    {
      annoTemplate.name = nameID.second;

      auto keysLower = db.nodeAnnoKeys.lower_bound({nameID.second, 0});
      auto keysUpper = db.nodeAnnoKeys.upper_bound({nameID.second, uintmax});
      for(auto itKey = keysLower; itKey != keysUpper; itKey++)
      {
        auto lowerAnno = db.inverseNodeAnnotations.lower_bound({itKey->name, itKey->ns, lowerVal});
        auto upperAnno = db.inverseNodeAnnotations.lower_bound({itKey->name, itKey->ns, upperVal});
        searchRanges.push_back(Range(lowerAnno, upperAnno));
      }
    }
  } // end if the regex is ok
  currentRange = searchRanges.begin();

  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
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
  currentRange = searchRanges.begin();
  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }
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
    while(currentRange != searchRanges.end())
    {
      while(it != currentRange->second)
      {
        Match candidate = {it->second, it->first};
        it++;

        if(RE2::FullMatch(db.strings.str(candidate.anno.val), compiledValRegex))
        {
          currentMatch = candidate;
          currentMatchValid = true;
          return;
        }
      } // end for each item in search range
      currentRange++;
    } // end for each search range
  }
}



