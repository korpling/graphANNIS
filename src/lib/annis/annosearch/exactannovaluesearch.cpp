#include <annis/annosearch/exactannovaluesearch.h>

using namespace annis;
using namespace std;

ExactAnnoValueSearch::ExactAnnoValueSearch(const DB &db, const string &annoNamspace, const string &annoName, const string &annoValue)
  :db(db),validAnnotationInitialized(false), debugDescription(annoNamspace + ":" + annoName + "=\"" + annoValue + "\"")
{
  std::pair<bool, uint32_t> nameID = db.strings.findID(annoName);
  std::pair<bool, uint32_t> namspaceID = db.strings.findID(annoNamspace);
  std::pair<bool, uint32_t> valueID = db.strings.findID(annoValue);

  if(nameID.first && namspaceID.first && valueID.first)
  {
    Annotation key;
    key.name = nameID.second;
    key.ns = namspaceID.second;
    key.val = valueID.second;

    searchRanges.push_back(Range(db.nodeAnnos.inverseAnnotations.equal_range(key)));
    it = searchRanges.begin()->first;
  }
  currentRange = searchRanges.begin();
}

ExactAnnoValueSearch::ExactAnnoValueSearch(const DB &db, const std::string &annoName, const std::string &annoValue)
  :db(db), validAnnotationInitialized(false), debugDescription(annoName + "=\"" + annoValue + "\"")
{
  std::pair<bool, uint32_t> nameID = db.strings.findID(annoName);
  std::pair<bool, uint32_t> valueID = db.strings.findID(annoValue);

  if(nameID.first && valueID.first)
  {
    auto keysLower = db.nodeAnnos.annoKeys.lower_bound({nameID.second, 0});
    auto keysUpper = db.nodeAnnos.annoKeys.upper_bound({nameID.second, uintmax});
    for(auto itKey = keysLower; itKey != keysUpper; itKey++)
    {
      searchRanges.push_back(Range(db.nodeAnnos.inverseAnnotations.equal_range(
      {itKey->first.name, itKey->first.ns, valueID.second})));
    }
  }
  currentRange = searchRanges.begin();

  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }
}

bool ExactAnnoValueSearch::next(Match& result)
{
  if(currentRange != searchRanges.end() && it != currentRange->second)
  {
    result.node = it->second; // node ID
    result.anno = it->first; // annotation itself
    it++;
    if(it == currentRange->second)
    {
      currentRange++;
      if(currentRange != searchRanges.end())
      {
        it = currentRange->first;
      }
    }
    return true;
  }
  else
  {
    return false;
  }
}

void ExactAnnoValueSearch::reset()
{
  currentRange = searchRanges.begin();
  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }
}

void ExactAnnoValueSearch::initializeValidAnnotations()
{
  for(auto range : searchRanges)
  {
    for(ItType annoIt = range.first; annoIt != range.second; annoIt++)
    {
      validAnnotations.insert(annoIt->first);
    }
  }

  validAnnotationInitialized = true;
}

std::int64_t ExactAnnoValueSearch::guessMaxCount() const
{
  std::int64_t sum = 0;
  
  for(auto range : searchRanges)
  {
    if(range.first != range.second)
    {
      const Annotation& anno = range.first->first;
      const std::string val = db.strings.str(anno.val);
      sum += db.nodeAnnos.guessMaxCount(anno.ns, anno.name, val, val);
    }
  }
  
  return sum;
}



ExactAnnoValueSearch::~ExactAnnoValueSearch()
{

}


