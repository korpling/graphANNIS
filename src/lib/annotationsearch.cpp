#include "annotationsearch.h"

using namespace annis;
using namespace std;

AnnotationNameSearch::AnnotationNameSearch(DB& db, const string& annoName)
  : db(db)
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

    it = db.inverseNodeAnnotations.lower_bound(lowerKey);
    itEnd = db.inverseNodeAnnotations.upper_bound(upperKey);
  }
  else
  {
    it = db.inverseNodeAnnotations.end();
    itEnd = db.inverseNodeAnnotations.end();
  }
}

AnnotationNameSearch::AnnotationNameSearch(DB &db, const string &annoNamspace, const string &annoName, const string &annoValue)
  :db(db)
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

    it = db.inverseNodeAnnotations.lower_bound(key);
    itEnd = db.inverseNodeAnnotations.upper_bound(key);
  }
  else
  {
    it = db.inverseNodeAnnotations.end();
    itEnd = db.inverseNodeAnnotations.end();
  }
}



Match AnnotationNameSearch::next()
{
  Match result;
  if(hasNext())
  {
    result.first = it->second; // node ID
    result.second = it->first; // annotation itself
    it++;
  }
  return result;
}
