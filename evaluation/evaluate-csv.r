rm(list = ls())
fn <- c("2016-07-19.csv")

d = read.csv(fn)
options(scipen = 5)
d <- subset(d, Experiment=='Optimized')

worse <- subset(d, Baseline >= 1.0)
better <- subset(d, Baseline < 1.0)

barplot(sort(d$Baseline), main="Baseline speedup distribution", log="y")
message("worse: ", nrow(worse), " better: ", nrow(better))
quantile(d$Baseline)

