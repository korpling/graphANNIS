#include "exactannosearch.h"

#include "db.h"

using namespace annis;
using namespace std;

ExactAnnoSearch::ExactAnnoSearch(const DB &db, const string &annoNamspace, const string &annoName, const string &annoValue)
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

    itBegin = db.inverseNodeAnnotations.lower_bound(key);
    it = itBegin;
    itEnd = db.inverseNodeAnnotations.upper_bound(key);
  }
  else
  {
    itBegin = db.inverseNodeAnnotations.end();
    it = itBegin;
    itEnd = db.inverseNodeAnnotations.end();
  }
}

Match ExactAnnoSearch::next()
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
  }
  return result;
}

void ExactAnnoSearch::reset()
{
  it = itBegin;
}

void ExactAnnoSearch::initializeValidAnnotations()
{
  for(ItType annoIt = itBegin; annoIt != itEnd; annoIt++)
  {
    validAnnotations.insert(annoIt->first);
  }
  validAnnotationInitialized = true;
}


ExactAnnoSearch::~ExactAnnoSearch()
{

}
