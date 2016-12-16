#pragma once

#include <cereal/cereal.hpp>
#include <annis/serializers.h>
#include <annis/types.h>

#include <boost/optional.hpp>

#include <annis/stringstorage.h>

namespace annis {


class AnnotationStatisticHolder
{
public:
  AnnotationStatisticHolder(StringStorage& strings);
  virtual ~AnnotationStatisticHolder();

  void clear();

  bool hasStatistics() const;
  void calculateStatistics();

  size_t estimateMemorySize();

  std::int64_t guessMaxCount(const std::string& ns, const std::string& name, const std::string& val) const;
  std::int64_t guessMaxCount(const std::string& name, const std::string& val) const;

  std::int64_t guessMaxCountRegex(const std::string& ns, const std::string& name, const std::string& val) const;
  std::int64_t guessMaxCountRegex(const std::string& name, const std::string& val) const;

  template <class Archive>
  void serialize( Archive & ar )
  {
    ar(histogramBounds);
  }

protected:

  virtual const btree::btree_map<AnnotationKey, std::uint64_t>& getAnnoKeys() const = 0;

  virtual std::vector<Annotation> getAnnotationRange(Annotation minAnno, Annotation maxAnno) = 0;

private:
  StringStorage& strings;
  btree::btree_map<AnnotationKey, std::vector<std::string>> histogramBounds;

protected:
  /**
   * Internal function for getting an estimation about the number of matches for a certain range of annotation value
   * @param nsID The namespace part of the annotation key. Can be empty (in this case all annotations with the correct name are used).
   * @param nameID The name part of the annotation key.
   * @param lowerVal Inclusive starting point for the value range.
   * @param upperVal Inclusive end point for the value range.
   * @param if true upperVal is inclusive, otherwise it is exclusive
   * @return The estimation of -1 if invalid.
   */
  std::int64_t guessMaxCount(boost::optional<std::uint32_t> nsID, std::uint32_t nameID, const std::string& lowerVal,
    const std::string& upperVal) const;
};

}
