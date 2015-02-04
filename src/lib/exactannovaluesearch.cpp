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

//ExactAnnoValueSearch::ExactAnnoValueSearch(const DB &db, const std::string &annoName, const std::string &annoValue)
//  :db(db), validAnnotationInitialized(false)
//{
//  std::pair<bool, uint32_t> nameID = db.strings.findID(annoName);
//  std::pair<bool, uint32_t> valueID = db.strings.findID(annoValue);
//}

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
  }
  return result;
}

void ExactAnnoValueSearch::reset()
{
  it = itBegin;
}

void ExactAnnoValueSearch::initializeValidAnnotations()
{
  for(ItType annoIt = itBegin; annoIt != itEnd; annoIt++)
  {
    validAnnotations.insert(annoIt->first);
  }
  validAnnotationInitialized = true;
}


ExactAnnoValueSearch::~ExactAnnoValueSearch()
{

}


