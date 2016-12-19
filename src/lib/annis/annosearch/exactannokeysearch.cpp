#include <annis/annosearch/exactannokeysearch.h>

#include <annis/db.h>

using namespace annis;
using namespace std;

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB &db)
  : db(db),
    validAnnotationKeysInitialized(false), debugDescription("node")
{
  itBegin = db.nodeAnnos.inverseAnnotations.begin();
  itEnd = db.nodeAnnos.inverseAnnotations.end();
  it = itBegin;

  itKeyBegin = db.nodeAnnos.annoKeys.begin();
  itKeyBegin = db.nodeAnnos.annoKeys.end();
}

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB& db, const string& annoName)
  : db(db),
    validAnnotationKeysInitialized(false), debugDescription(annoName)
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

    itBegin = db.nodeAnnos.inverseAnnotations.lower_bound(lowerKey);
    it = itBegin;
    itEnd = db.nodeAnnos.inverseAnnotations.upper_bound(upperKey);

    itKeyBegin = db.nodeAnnos.annoKeys.lower_bound({searchResult.second, 0});
    itKeyEnd = db.nodeAnnos.annoKeys.upper_bound({searchResult.second, uintmax});
  }
  else
  {
    itBegin = itEnd = it = db.nodeAnnos.inverseAnnotations.end();
    itKeyBegin = itKeyEnd = db.nodeAnnos.annoKeys.end();
  }
}

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB &db, const string &annoNamspace, const string &annoName)
  : db(db),
    validAnnotationKeysInitialized(false), debugDescription(annoNamspace + ":" + annoName)
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

    itBegin = db.nodeAnnos.inverseAnnotations.lower_bound(lowerKey);
    it = itBegin;
    itEnd = db.nodeAnnos.inverseAnnotations.upper_bound(upperKey);

    itKeyBegin = db.nodeAnnos.annoKeys.lower_bound({nameID.second, namespaceID.second});
    itKeyEnd = db.nodeAnnos.annoKeys.upper_bound({nameID.second, namespaceID.second});
  }
  else
  {
    itBegin = itEnd = it = db.nodeAnnos.inverseAnnotations.end();
    itKeyBegin = itKeyEnd = db.nodeAnnos.annoKeys.end();
  }
}

bool ExactAnnoKeySearch::next(Match& result)
{
  if(it != db.nodeAnnos.inverseAnnotations.end() && it != itEnd)
  {
    result.node = it->second; // node ID
    result.anno = it->first; // annotation itself
    it++;
    return true;
  }
  else
  {
    return false;
  }
}

void ExactAnnoKeySearch::reset()
{
  it = itBegin;
}

void ExactAnnoKeySearch::initializeValidAnnotationKeys()
{
  for(ItAnnoKey itKey = itKeyBegin; itKey != itKeyEnd; itKey++)
  {
    validAnnotationKeys.insert(itKey->first);
  }
  validAnnotationKeysInitialized = true;
}

std::int64_t ExactAnnoKeySearch::guessMaxCount() const
{ 
  std::int64_t sum = 0;
  for(auto itKey = itKeyBegin; itKey != itKeyEnd; itKey++)
  {
    sum += itKey->second;
  }
  return sum;
}


ExactAnnoKeySearch::~ExactAnnoKeySearch()
{

}
