#include <annis/annosearch/regexannosearch.h>
#include <limits>

using namespace annis;

RegexAnnoSearch::RegexAnnoSearch(const DB &db, const std::string& ns,
                                 const std::string& name, const std::string& valRegex)
  : db(db),
    validAnnotationsInitialized(false), valRegex(valRegex),
    compiledValRegex(valRegex)
{
  std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
  std::pair<bool, std::uint32_t> namespaceID = db.strings.findID(ns);
  if(nameID.first && namespaceID.first)
  {
    annoTemplates.push_back({nameID.second, namespaceID.second, 0});
    
    auto lower = db.nodeAnnos.inverseNodeAnnotations.lower_bound({nameID.second, namespaceID.second, 0});
    auto upper = db.nodeAnnos.inverseNodeAnnotations.lower_bound({nameID.second, namespaceID.second, uintmax});
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
    compiledValRegex(valRegex)
{
  if(compiledValRegex.ok())
  {
    std::pair<bool, std::uint32_t> nameID = db.strings.findID(name);
    if(nameID.first)
    {
      auto keysLower = db.nodeAnnos.nodeAnnoKeys.lower_bound({nameID.second, 0});
      auto keysUpper = db.nodeAnnos.nodeAnnoKeys.upper_bound({nameID.second, uintmax});
      for(auto itKey = keysLower; itKey != keysUpper; itKey++)
      {
        annoTemplates.push_back({itKey->name, itKey->ns, 0});
        
        auto lowerAnno = db.nodeAnnos.inverseNodeAnnotations.lower_bound({itKey->name, itKey->ns, 0});
        auto upperAnno = db.nodeAnnos.inverseNodeAnnotations.lower_bound({itKey->name, itKey->ns, uintmax});
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

bool RegexAnnoSearch::next(Match& result)
{
  if(compiledValRegex.ok())
  {
    while(currentRange != searchRanges.end())
    {
      while(it != currentRange->second)
      {
        if(RE2::FullMatch(db.strings.str(it->first.val), compiledValRegex))
        {
          result = {it->second, it->first};
          it++;
          return true;
        }
        // skip to the next available key (we don't want to iterate over each value of the multimap)
        it = db.nodeAnnos.inverseNodeAnnotations.upper_bound(it->first);

      } // end for each item in search range
      currentRange++;
      if(currentRange != searchRanges.end())
      {
        it = currentRange->first;
      }
    } // end for each search range
  }
  
  return false;
}

void RegexAnnoSearch::reset()
{
  currentRange = searchRanges.begin();
  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
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
    for(auto annoCopy : annoTemplates)
    {
      annoCopy.val = id;
      validAnnotations.insert(annoCopy);
    }
  }

  validAnnotationsInitialized = true;
}

std::int64_t RegexAnnoSearch::guessMaxCount() const
{
  std::int64_t sum = 0;
  
  for(const auto& anno : annoTemplates)
  {
    sum += db.nodeAnnos.guessMaxCountRegex(db.strings.str(anno.ns), db.strings.str(anno.name), valRegex);
  }
  
  return sum;
}



