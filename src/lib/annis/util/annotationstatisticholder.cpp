#include "annotationstatisticholder.h"

#include <annis/util/size_estimator.h>

#include <re2/re2.h>
#include <cmath>


using namespace annis;

AnnotationStatisticHolder::AnnotationStatisticHolder(StringStorage& strings)
  : strings(strings)
{

}

AnnotationStatisticHolder::~AnnotationStatisticHolder()
{

}

void AnnotationStatisticHolder::clear()
{
  histogramBounds.clear();
}

bool AnnotationStatisticHolder::hasStatistics() const
{
  return !histogramBounds.empty();
}


void AnnotationStatisticHolder::calculateStatistics()
{

  const size_t maxHistogramBuckets = 250;
  const size_t maxSampledAnnotations = 2500;

  histogramBounds.clear();

  // collect statistics for each annotation key separatly
  std::map<AnnotationKey, std::vector<std::string>> globalValueList;
  for(const auto& annoKey : getAnnoKeys())
  {
    histogramBounds[annoKey.first] = std::vector<std::string>();
    auto& valueList = globalValueList[annoKey.first] = std::vector<std::string>();

    // get all annotations
    Annotation minAnno = {annoKey.first.name, annoKey.first.ns, 0};
    Annotation maxAnno = {annoKey.first.name, annoKey.first.ns, std::numeric_limits<std::uint32_t>::max()};
    std::vector<Annotation> annos = getAnnotationRange(minAnno, maxAnno);

    std::random_shuffle(annos.begin(), annos.end());
    valueList.resize(std::min<size_t>(maxSampledAnnotations, annos.size()));
    for(size_t i=0; i < valueList.size(); i++)
    {
      valueList[i] = strings.str(annos[i].val);
    }

  }

  // create uniformly distributed histogram bounds for each node annotation key
  for(auto it=globalValueList.begin(); it != globalValueList.end(); it++)
  {
    auto& values = it->second;

    std::sort(values.begin(), values.end());

    size_t numValues = values.size();

    size_t numHistBounds = maxHistogramBuckets + 1;
    if(numValues < numHistBounds)
    {
      numHistBounds = numValues;
    }

    if(numHistBounds >= 2)
    {
      auto& h = histogramBounds[it->first];
      h.resize(numHistBounds);

      std::int64_t delta = (numValues-1) / (numHistBounds -1);
      std::int64_t deltaFraction = (numValues -1) % (numHistBounds - 1);

    std::int64_t pos = 0;
    size_t posFraction = 0;
      for(size_t i=0; i < numHistBounds; i++)
      {
        h[i] = values[pos];
        pos += delta;
        posFraction += deltaFraction;

        if(posFraction >= (numHistBounds - 1))
        {
          pos++;
          posFraction -= (numHistBounds - 1);
        }
      }
    }
  }
}

size_t AnnotationStatisticHolder::estimateMemorySize()
{
  return size_estimation::element_size(histogramBounds);
}

std::int64_t AnnotationStatisticHolder::guessMaxCount(const std::string& ns, const std::string& name, const std::string& val) const
{
  auto nameID = strings.findID(name);
  if(nameID.first)
  {
    auto nsID = strings.findID(ns);
    if(nsID.first)
    {
      return guessMaxCount(boost::optional<std::uint32_t>(nsID.second), nameID.second,
        val, val);
    }
  }


  // if none of the conditions above is valid the annotation key does not exist
  return 0;
}

std::int64_t AnnotationStatisticHolder::guessMaxCount(const std::string& name, const std::string& val) const
{
  auto nameID = strings.findID(name);
  if(nameID.first)
  {
    return guessMaxCount(boost::optional<std::uint32_t>(), nameID.second, val, val);
  }
  return 0;
}

std::int64_t AnnotationStatisticHolder::guessMaxCountRegex(const std::string& ns, const std::string& name, const std::string& val) const
{
  auto nameID = strings.findID(name);
  if(nameID.first)
  {
    auto nsID = strings.findID(ns);
    if(nsID.first)
    {
      re2::RE2 pattern(val);
      if(pattern.ok())
      {
        std::string minMatch;
        std::string maxMatch;
        pattern.PossibleMatchRange(&minMatch, &maxMatch, 10);
        return guessMaxCount(boost::optional<std::uint32_t>(nsID.second), nameID.second, minMatch, maxMatch);
      }
    }
  }

  return 0;
}

std::int64_t AnnotationStatisticHolder::guessMaxCountRegex(const std::string& name, const std::string& val) const
{
  auto nameID = strings.findID(name);
  if(nameID.first)
  {
    re2::RE2 pattern(val);
    if(pattern.ok())
    {
      std::string minMatch;
      std::string maxMatch;
      pattern.PossibleMatchRange(&minMatch, &maxMatch, 10);
      return guessMaxCount(boost::optional<std::uint32_t>(), nameID.second, minMatch, maxMatch);
    }
  }
  return 0;
}


std::int64_t AnnotationStatisticHolder::guessMaxCount(boost::optional<std::uint32_t> nsID,
  std::uint32_t nameID,
  const std::string& lowerVal, const std::string& upperVal) const
{

  btree::btree_map<AnnotationKey, std::uint64_t>::const_iterator itBegin;
  btree::btree_map<AnnotationKey, std::uint64_t>::const_iterator itEnd;
  if(nsID)
  {
     itBegin = getAnnoKeys().lower_bound({nameID, *nsID});
     itEnd = getAnnoKeys().upper_bound({nameID, *nsID});
  }
  else
  {
    // find all complete keys which have the given name
     itBegin = getAnnoKeys().lower_bound({nameID, 0});
     itEnd = getAnnoKeys().upper_bound({nameID, std::numeric_limits<std::uint32_t>::max()});
  }

  std::int64_t universeSize = 0;
  std::int64_t sumHistogramBuckets = 0;
  std::int64_t countMatches = 0;
  // guess for each annotation fully qualified key and return the sum of all guesses
  for(auto itKeyCount = itBegin; itKeyCount != itEnd; itKeyCount++)
  {
    universeSize += itKeyCount->second;

    auto itHisto = histogramBounds.find(itKeyCount->first);
    if(itHisto != histogramBounds.end())
    {
      // find the range in which the value is contained
      const auto& histo = itHisto->second;

      // we need to make sure the histogram is not empty -> should have at least two bounds
      if(histo.size() >= 2)
      {
        sumHistogramBuckets += (histo.size() - 1);

        for(size_t i = 0; i < (histo.size()-1); i++)
        {
          const auto& bucketBegin = histo[i];
          const auto& bucketEnd = histo[i+1];
          // check if the range overlaps with the search range
          if(bucketBegin <= upperVal && lowerVal <= bucketEnd)
          {
            countMatches++;
          }
        }
      }
    }
  }

  if(sumHistogramBuckets > 0)
  {
    double selectivity = ((double) countMatches) / ((double) sumHistogramBuckets);
    return std::round(selectivity * ((double) universeSize));
  }
  else
  {
    return 0;
  }

}
