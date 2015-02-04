#include "exactannovaluesearch.h"

#include "db.h"

using namespace annis;
using namespace std;

ExactAnnoValueSearch::ExactAnnoValueSearch(const DB &db, const string &annoNamspace, const string &annoName, const string &annoValue)
  :db(db),validAnnotationInitialized(false)
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

    searchRanges.push_back(Range(db.inverseNodeAnnotations.equal_range(key)));
    it = searchRanges.begin()->first;
  }
  currentRange = searchRanges.begin();
}

ExactAnnoValueSearch::ExactAnnoValueSearch(const DB &db, const std::string &annoName, const std::string &annoValue)
  :db(db), validAnnotationInitialized(false)
{
  std::pair<bool, uint32_t> nameID = db.strings.findID(annoName);
  std::pair<bool, uint32_t> valueID = db.strings.findID(annoValue);

  if(nameID.first && valueID.first)
  {
    auto keysLower = db.nodeAnnoKeys.lower_bound({nameID.second, 0});
    auto keysUpper = db.nodeAnnoKeys.upper_bound({nameID.second, uintmax});
    for(auto itKey = keysLower; itKey != keysUpper; itKey++)
    {
      searchRanges.push_back(Range(db.inverseNodeAnnotations.equal_range(
      {itKey->name, itKey->ns, valueID.second})));
    }
  }
  currentRange = searchRanges.begin();

  if(currentRange != searchRanges.end())
  {
    it = currentRange->first;
  }
}

Match ExactAnnoValueSearch::next()
{
  Match result;
  currentMatchValid = false;
  if(hasNext())
  {
    result.node = it->second; // node ID
    result.anno = it->first; // annotation itself
    currentMatch = result;
    currentMatchValid = true;
    it++;
    if(it == currentRange->second)
    {
      currentRange++;
      if(currentRange != searchRanges.end())
      {
        it = currentRange->first;
      }
    }
  }
  return result;
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


ExactAnnoValueSearch::~ExactAnnoValueSearch()
{

}


