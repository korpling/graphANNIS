#include "regexannosearch.h"
#include <limits>

using namespace annis;

RegexAnnoSearch::RegexAnnoSearch(const DB &db, const std::string& ns,
                                 const std::string& name, const std::string& valRegex)
  : db(db),
    validAnnotationsInitialized(false), valRegex(valRegex),
    compiledValRegex(valRegex),
    currentMatchValid(false)
{
  std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
  std::pair<bool, std::uint32_t> namespaceID = db.strings.findID(ns);
  if(nameID.first && namespaceID.first)
  {
    annoTemplates.push_back({nameID.second, namespaceID.second, 0});
    
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
    currentMatchValid(false)
{
  if(compiledValRegex.ok())
  {
    std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
    if(nameID.first)
    {
      auto keysLower = db.nodeAnnoKeys.lower_bound({nameID.second, 0});
      auto keysUpper = db.nodeAnnoKeys.upper_bound({nameID.second, uintmax});
      for(auto itKey = keysLower; itKey != keysUpper; itKey++)
      {
        annoTemplates.push_back({itKey->name, itKey->ns, 0});
        
        auto lowerAnno = db.inverseNodeAnnotations.lower_bound({itKey->name, itKey->ns, 0});
        auto upperAnno = db.inverseNodeAnnotations.lower_bound({itKey->name, itKey->ns, uintmax});
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
    for(auto annoCopy : annoTemplates)
    {
      annoCopy.val = id;
      validAnnotations.insert(annoCopy);
    }
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
        if(RE2::FullMatch(db.strings.str(it.key().val), compiledValRegex))
        {
          currentMatch = {it.data(), it.key()};
          currentMatchValid = true;
          it++;
          return;
        }
        // skip to the next available key (we don't want to iterate over each value of the multimap)
        it = db.inverseNodeAnnotations.upper_bound(it.key());

      } // end for each item in search range
      currentRange++;
      if(currentRange != searchRanges.end())
      {
        it = currentRange->first;
      }
    } // end for each search range
  }
}



