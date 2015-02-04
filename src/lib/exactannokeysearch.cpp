#include "exactannokeysearch.h"

#include "db.h"

using namespace annis;
using namespace std;

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB &db)
  : db(db), currentMatchValid(false),
    validAnnotationKeysInitialized(false)
{
  itBegin = db.inverseNodeAnnotations.begin();
  itEnd = db.inverseNodeAnnotations.end();
  it = itBegin;

  itKeyBegin = db.nodeAnnoKeys.begin();
  itKeyBegin = db.nodeAnnoKeys.end();
}

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB& db, const string& annoName)
  : db(db), currentMatchValid(false),
    validAnnotationKeysInitialized(false)
{
  std::pair<bool, uint32_t> searchResult = db.strings.findID(annoName);

  if(searchResult.first)
  {
    Annotation lowerKey;
    lowerKey.name = searchResult.second;
    lowerKey.ns = numeric_limits<uint32_t>::min();
    lowerKey.val = numeric_limits<uint32_t>::min();

    Annotation upperKey;
    upperKey.name = searchResult.second;
    upperKey.ns = numeric_limits<uint32_t>::max();
    upperKey.val = numeric_limits<uint32_t>::max();

    itBegin = db.inverseNodeAnnotations.lower_bound(lowerKey);
    it = itBegin;
    itEnd = db.inverseNodeAnnotations.upper_bound(upperKey);

    itKeyBegin = db.nodeAnnoKeys.lower_bound({searchResult.second, 0});
    itKeyEnd = db.nodeAnnoKeys.upper_bound({searchResult.second, uintmax});
  }
  else
  {
    itBegin = itEnd = it = db.inverseNodeAnnotations.end();
    itKeyBegin = itKeyEnd = db.nodeAnnoKeys.end();
  }
}

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB &db, const string &annoNamspace, const string &annoName)
  : db(db), currentMatchValid(false),
    validAnnotationKeysInitialized(false)
{
  std::pair<bool, uint32_t> nameID = db.strings.findID(annoName);
  std::pair<bool, uint32_t> namespaceID = db.strings.findID(annoNamspace);

  if(nameID.first && namespaceID.first)
  {
    Annotation lowerKey;
    lowerKey.name = nameID.second;
    lowerKey.ns = namespaceID.second;
    lowerKey.val = numeric_limits<uint32_t>::min();

    Annotation upperKey;
    upperKey.name = nameID.second;
    upperKey.ns = namespaceID.second;
    upperKey.val = numeric_limits<uint32_t>::max();

    itBegin = db.inverseNodeAnnotations.lower_bound(lowerKey);
    it = itBegin;
    itEnd = db.inverseNodeAnnotations.upper_bound(upperKey);

    itKeyBegin = db.nodeAnnoKeys.lower_bound({nameID.second, namespaceID.second});
    itKeyEnd = db.nodeAnnoKeys.upper_bound({nameID.second, namespaceID.second});
  }
  else
  {
    itBegin = itEnd = it = db.inverseNodeAnnotations.end();
    itKeyBegin = itKeyEnd = db.nodeAnnoKeys.end();
  }
}

Match ExactAnnoKeySearch::next()
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

void ExactAnnoKeySearch::reset()
{
  it = itBegin;
}

void ExactAnnoKeySearch::initializeValidAnnotationKeys()
{
  for(ItAnnoKey itKey = itKeyBegin; itKey != itKeyEnd; itKey++)
  {
    validAnnotationKeys.insert(*itKey);
  }
  validAnnotationKeysInitialized = true;
}

ExactAnnoKeySearch::~ExactAnnoKeySearch()
{

}
