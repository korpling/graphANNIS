bench_getaql <- function(x, querydir, named=FALSE) {
  aqlFile <- ""
  group <- x[1]
  problemSpace = as.numeric(x[2])
  if(named && problemSpace == 0) {
    corpus <- sub("_[^_]+$", "", group)
    fn <- substr(group, nchar(corpus)+2, nchar(group))
    
    aqlFile <- paste(querydir, "/", corpus,"/", fn, ".aql", sep='')
  } else {
    aqlFile <- paste(querydir, "/", x[1] ,"/", formatC(problemSpace, width=5, flag="0",  mode="integer"), ".aql", sep='')
  }
  
  aql <- readChar(aqlFile, file.info(aqlFile)$size)
  
  return(aql)
}
bench_extract <- function(fn, querydir) {
  d = read.csv(fn)
  d <- subset(d, Experiment=='Optimized')
  
  if(!missing(querydir)) {
    # try to get the original AQL queries
    d$aql <- apply(d[,c("Group", "Problem.Space")], 1, bench_getaql, querydir=querydir)
  } else {
    d$aql <- ""
  }
  
  
  
  return(d)
}

bench_desc <- function(d) {
  
  worse <- subset(d, Baseline >= 1.0)
  better <- subset(d, Baseline < 1.0)
  
  q <- quantile(d$Baseline)
  
  return (list(worse=nrow(worse), better=nrow(better), quantile=q, 
               sumTime=sum(d$us.Iteration)))
}

bench_plot <- function(d, header=NULL) {
  h <- sort(1.0/ d$Baseline)
  plot(h, log="y", type="p", pch="*", xaxt="n", ylim=c(min(h),max(h) * 10), 
       xlab="query",
       ylab="times faster than baseline", main=header)
  grid()
  lines(x=c(0, nrow(d)), y=c(1.0, 1.0), col="red")
}





