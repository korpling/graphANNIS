#include "annotationsearch.h"

using namespace annis;
using namespace std;

AnnotationNameSearch::AnnotationNameSearch(DB& db, string annoName)
  : db(db), annoName(annoName)
{
  std::pair<bool, uint32_t> searchResult = db.findString(annoName);

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
