#include <annis/annosearch/exactannokeysearch.h>

#include <annis/db.h>

using namespace annis;
using namespace std;

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB &db)
  : db(db),
    validAnnotationKeysInitialized(false)
{
  itBegin = db.nodeAnnos.inverseNodeAnnotations.begin();
  itEnd = db.nodeAnnos.inverseNodeAnnotations.end();
  it = itBegin;

  itKeyBegin = db.nodeAnnos.nodeAnnoKeys.begin();
  itKeyBegin = db.nodeAnnos.nodeAnnoKeys.end();
}

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB& db, const string& annoName)
  : db(db),
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

    itBegin = db.nodeAnnos.inverseNodeAnnotations.lower_bound(lowerKey);
    it = itBegin;
    itEnd = db.nodeAnnos.inverseNodeAnnotations.upper_bound(upperKey);

    itKeyBegin = db.nodeAnnos.nodeAnnoKeys.lower_bound({searchResult.second, 0});
    itKeyEnd = db.nodeAnnos.nodeAnnoKeys.upper_bound({searchResult.second, uintmax});
  }
  else
  {
    itBegin = itEnd = it = db.nodeAnnos.inverseNodeAnnotations.end();
    itKeyBegin = itKeyEnd = db.nodeAnnos.nodeAnnoKeys.end();
  }
}

ExactAnnoKeySearch::ExactAnnoKeySearch(const DB &db, const string &annoNamspace, const string &annoName)
  : db(db),
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

    itBegin = db.nodeAnnos.inverseNodeAnnotations.lower_bound(lowerKey);
    it = itBegin;
    itEnd = db.nodeAnnos.inverseNodeAnnotations.upper_bound(upperKey);

    itKeyBegin = db.nodeAnnos.nodeAnnoKeys.lower_bound({nameID.second, namespaceID.second});
    itKeyEnd = db.nodeAnnos.nodeAnnoKeys.upper_bound({nameID.second, namespaceID.second});
  }
  else
  {
    itBegin = itEnd = it = db.nodeAnnos.inverseNodeAnnotations.end();
    itKeyBegin = itKeyEnd = db.nodeAnnos.nodeAnnoKeys.end();
  }
}

bool ExactAnnoKeySearch::next(Match& result)
{
  if(it != db.nodeAnnos.inverseNodeAnnotations.end() && it != itEnd)
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
    validAnnotationKeys.insert(*itKey);
  }
  validAnnotationKeysInitialized = true;
}

std::int64_t ExactAnnoKeySearch::guessMaxCount() const
{ 
  std::int64_t sum = 0;
  for(auto itKey = itKeyBegin; itKey != itKeyEnd; itKey++)
  {
    auto itCount = db.nodeAnnos.nodeAnnotationKeyCount.find(*itKey);
    if(itCount != db.nodeAnnos.nodeAnnotationKeyCount.end())
    {
      sum += itCount->second;
    }
  }
  return sum;
}


ExactAnnoKeySearch::~ExactAnnoKeySearch()
{

}
