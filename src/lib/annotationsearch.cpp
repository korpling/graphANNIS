#include "annotationsearch.h"

using namespace annis;

AnnotationNameSearch::AnnotationNameSearch(DB& db, std::string annoName)
  : db(db), annoName(annoName)
{
}

bool AnnotationNameSearch::hasNext()
{
  return false;
}

Match AnnotationNameSearch::next()
{

}
