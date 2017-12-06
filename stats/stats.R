library(ggplot2)
library(dRank)
library(reshape2)

gm_mean = function(x) {
	exp(mean(log(x)))
}

se = function(x) {
	sd(x)/sqrt(length(x))
}

tech_scores <- seq(0.97, 0.27, -(0.97 - 0.27) / 11)
tech_weights <- tech_scores / sum(tech_scores)
tech_names <- c("0.97", "0.91", "0.84", "0.78", "0.72", "0.65", "0.59", "0.53", "0.46", "0.40", "0.34", "0.27")
tech <- data.frame(sample = tech_names, score = tech_scores, weight = tech_weights)

cw_calculate = function(file) {
	df <- read.csv(file, header = TRUE, colClasses = c("item_name" = "character"))

	samples <- unique(df$item_name)
	users <- unique(df$user)

	weight_gm <- aggregate(x = df[c("weight")], by = list(sample = df$item_name), FUN = gm_mean)
	weight_am <- aggregate(x = df[c("weight")], by = list(sample = df$item_name), FUN = mean)
	weight_se <- aggregate(x = df[c("weight")], by = list(sample = df$item_name), FUN = se)
	weight_sd <- aggregate(x = df[c("weight")], by = list(sample = df$item_name), FUN = sd)
	mean <- data.frame(sample = weight_gm$sample, gm = weight_gm$weight, am = weight_am$weight, sd = weight_sd$weight, se = weight_se$weight)
	mean <- mean[with(mean, order(-gm)), ]
	mean$rank <- 1:length(samples)

	list(
		df = df,
		samples = samples,
		users = users,
		mean = mean
	)
}

cw_stats = function(file) {
	cw <- cw_calculate(file)

	cat("Num samples:", length(cw$samples), "\n")
	cat("Num participants:", length(cw$users), "\n")

	ranking_human <- cw$mean[with(cw$mean, rev(order(sample))), ]$gm
	ranking_code <- tech_weights

	print(cw$mean[cw$mean$sample == '0.97', ]$gm - cw$mean[cw$mean$sample == '0.84', ]$gm)
	print(cw$mean[cw$mean$sample == '0.46', ]$gm - cw$mean[cw$mean$sample == '0.40', ]$gm)

	print(cor.test(
		ranking_human,
		ranking_code,
		method = "kendall",
		alternative = "greater"
	))

	tmp <- cw$df[with(cw$df, order(item_name, user)), ]
	X <- matrix(
		tmp$weight,
		nrow = length(cw$users),
		ncol = length(cw$samples),
	)
	y <- sort(ranking_code)
	print(dRank(y, X, B = 10000))
}

cw_plot_weights = function(file) {
	cw <- cw_calculate(file)

	data <- merge(cw$mean, tech, by = "sample")
	names(data)[names(data) == 'weight'] <- 'f'

	# weights <- melt(data[, c("sample", "gm", "am", "f")], id = "sample")
	weights <- melt(data[, c("sample", "gm", "f")], id = "sample")
	colnames(weights) <- c("sample", "type", "weight")

	sd <- melt(data[, c("sample", "sd")], id = "sample")
	sd$variable <- NULL
	colnames(sd) <- c("sample", "sd")
	sd$type = "gm"

	se <- melt(data[, c("sample", "se")], id = "sample")
	se$variable <- NULL
	colnames(se) <- c("sample", "se")
	se$type = "gm"

	data <- merge(weights, sd, by = c("sample", "type"), all = TRUE)
	data <- merge(data, se, by = c("sample", "type"), all = TRUE)
	print(data)

	ggplot(data, aes(shape = type, color = type)) +
		geom_errorbar(aes(sample, ymin = weight - se, ymax = weight + se), width = 0.2) +
		geom_point(aes(sample, weight), size = 2.0)
}

cw_plot_all_weights = function(file) {
	cw <- cw_calculate(file)

	ggplot() +
		geom_point(data = cw$df, aes(item_name, weight), size = 0.5, alpha = 0.2, color = "black") +
		geom_point(data = cw$mean, aes(sample, gm), size = 2.0, alpha = 1.0, color = "blue3") +
		geom_point(data = tech, aes(sample, weight), size = 2.0, alpha = 1.0, color = "red3")
}

cw_plot_aggregate = function(file) {
	cw <- cw_calculate(file)
	# Order bars by mean weight
	cw$mean$sample <- factor(cw$mean$sample, levels = cw$mean$sample)

	ggplot(
		data=cw$mean,
		aes(sample, gm)
	) +
		ylab("weight geometric mean") +
		geom_bar(stat="identity") +
		geom_errorbar(aes(sample, ymin = gm - se, ymax = gm + se), width = 0.2)
}

cw_plot_fitness = function(cw_file, fitness_file) {
	cw <- cw_calculate(cw_file)
	data <- cw$mean
	names(data)[names(data) == 'gm'] <- 'human'
	data <- melt(data[, c("sample", "human")], id = "sample")
	colnames(data) <- c("sample", "evaluation", "weight")

	fitness <- read.csv(fitness_file, header = TRUE, colClasses = c("sample" = "character"))
	# fitness <- fitness[fitness$evaluation %in% c('default', 'removal2', 'only-balance', 'only-curvature'), ]
	fitness <- fitness[fitness$evaluation %in% c('default', 'removal'), ]
	# fitness <- fitness[fitness$evaluation %in% c('default', 'removal', 'only-balance', 'only-curvature', 'only-length', 'only-foliage'), ]
	# fitness <- fitness[fitness$evaluation %in% c('default', 'only-balance', 'only-branching', 'only-closeness', 'only-curvature', 'only-length', 'only-foliage', 'only-drop'), ]
	evaluations <- unique(fitness$evaluation)
	fitness$weight <- NA

	for (evaluation in evaluations) {
		normalized <- fitness[fitness$evaluation == evaluation, ]
		normalized$weight <- normalized$fitness / sum(normalized$fitness)
		normalized$fitness <- NULL
		fitness[fitness$evaluation == evaluation, ]$weight <- normalized$weight
	}

	fitness$fitness <- NULL
	fitness <- fitness[, c('sample', 'evaluation', 'weight')]

	data <- rbind(data, fitness)

	# Order bars by mean weight
	data <- dcast(data, sample ~ evaluation, value.var = 'weight')
	data <- data[with(data, order(-human)), ]
	data$sample <- factor(data$sample, levels = data$sample)
	data <- melt(data, id = "sample")
	colnames(data) <- c("sample", "evaluation", "weight")

	data <- transform(data, rank = as.numeric(sample))

	ggplot(
		data = data,
		aes(rank, weight)
	) +
		ylab("weight") +
		# geom_bar(aes(fill = evaluation), stat = "identity", position = "dodge")
		geom_point(aes(color = evaluation)) +
		# geom_line(aes(color = evaluation))
		geom_smooth(aes(color = evaluation), se = FALSE, span = 0.45) +
		scale_x_continuous(breaks = 1:12)
}

cw_fitness_correlation = function(cw_file, fitness_file) {
	cw <- cw_calculate(cw_file)
	data <- cw$mean
	names(data)[names(data) == 'gm'] <- 'human'
	data <- melt(data[, c("sample", "human")], id = "sample")
	colnames(data) <- c("sample", "evaluation", "weight")

	fitness <- read.csv(fitness_file, header = TRUE, colClasses = c("sample" = "character"))
	evaluations <- unique(fitness$evaluation)

	ranking_human <- cw$mean[with(cw$mean, rev(order(sample))), ]$gm

	tmp <- cw$df[with(cw$df, order(item_name, user)), ]
	X <- matrix(
		tmp$weight,
		nrow = length(cw$users),
		ncol = length(cw$samples),
	)

	correlations <- data.frame(metric = evaluations, tau = NA, tau.p = NA, d = NA, d.p = NA)

	for (evaluation in evaluations) {
		normalized <- fitness[fitness$evaluation == evaluation, ]
		normalized$weight <- normalized$fitness / sum(normalized$fitness)
		normalized$fitness <- NULL

		ranking_code <- normalized[with(normalized, rev(order(sample))), ]$weight

		kendall <- cor.test(ranking_human,
			ranking_code,
			method = "kendall",
			alternative = "greater"
		)

		correlations[correlations$metric == evaluation, ]$tau <- kendall$estimate
		correlations[correlations$metric == evaluation, ]$tau.p <- kendall$p.value

		y <- sort(ranking_code)
		drank <- dRank(y, X, B = 100)

		correlations[correlations$metric == evaluation, ]$d <- drank$dist
		correlations[correlations$metric == evaluation, ]$d.p <- drank$p
	}

	print(correlations[with(correlations, order(-tau, d)), ])
}


completed_users = function(users) {
	users[users$complete == "true", ]
}

user_stats = function(file) {
	users <- read.csv(file, header = TRUE)
	total <- nrow(users)
	users <- completed_users(users)

	completed <- nrow(users)
	cat(completed, ' of ', total, ' (', completed / total * 100, '%) completed the ranking\n')

	completed_post <- nrow(users[users$post == 'true', ])
	cat(completed_post, ' of ', completed, ' (', completed_post / completed * 100, '%) completed the post questionnaire\n')

	num_high <- nrow(users[users$education %in% c('bachelor', 'master', 'doctoral'), ])
	cat('Higher education (>=bachelor): ', num_high / completed * 100, '%\n')

	cat('Males: ', nrow(users[users$gender == 'male', ]) / completed * 100, '%\n')
	cat('IT: ', nrow(users[users$occupation == 'information_and_communication_technology', ]) / completed * 100, '%\n')
	cat('Science: ', nrow(users[users$occupation == 'science_and_engineering', ]) / completed * 100, '%\n')
	cat('Service: ', nrow(users[users$occupation == 'service_and_sales', ]) / completed * 100, '%\n')
	cat('Age 20-30: ', nrow(users[users$age >= 20 & users$age <= 30, ]) / completed * 100, '%\n')
	cat('Chrome: ', nrow(users[users$browser.name == 'chrome', ]) / completed * 100, '%\n')

	num_2nd <- nrow(users[users$from != '', ])
	cat('2nd: ', num_2nd, ' (', num_2nd / completed * 100, '%)\n')
}

user_gender = function(file) {
	users <- read.csv(file, header = TRUE)
	users <- completed_users(users)

	ggplot(
		data = users,
		aes(gender, ..count..)
	) +
		geom_bar()
}

user_education = function(file) {
	users <- read.csv(file, header = TRUE)
	users <- completed_users(users)

	ggplot(
		data = users,
		aes(education, ..count..)
	) +
		geom_bar()
}

user_occupation = function(file) {
	users <- read.csv(file, header = TRUE)
	users <- completed_users(users)

	ggplot(
		data = users,
		aes(occupation, ..count..)
	) +
		geom_bar()
}

user_age = function(file) {
	users <- read.csv(file, header = TRUE)
	users <- completed_users(users)

  mean <- data.frame(label = "mean", val = mean(users$age, na.rm = T))
  median <- data.frame(label = "median", val = median(users$age, na.rm = T))
  averages <- rbind(mean, median)

	# ggplot(
	# 	data = users,
	# 	aes(age)
	# ) +
  #   geom_histogram(aes(y = ..density..), colour="black", fill="white") +
  #   geom_density(alpha = 0.2, fill = "#FF6666") +
  #   geom_vline(data = averages, aes(xintercept = val, linetype = label, color = label), show.legend = TRUE)

	ggplot(
		data = users,
		aes(age, ..count..)
	) +
		geom_bar()
}

user_browser = function(file) {
	users <- read.csv(file, header = TRUE)
	users <- completed_users(users)

	ggplot(
		data = users,
		aes(browser.name, ..count..)
	) +
		geom_bar()
}

quest_stats = function(file) {
	q <- read.csv(file, header = TRUE)
	cat('Strongly agree: ', nrow(q[q$ranking_agree == 2, ]) / nrow(q) * 100, '%\n')
}

quest_plant_work = function(file) {
	q <- read.csv(file, header = TRUE)

	ggplot(
		data = q,
		aes(plant_work, ..count..)
	) +
		geom_bar() +
		scale_x_continuous(limits = c(-2.5, 2.5), breaks = seq(from = -2, to = 2, by = 1), labels = c('never', 'rarely', 'occationally', 'frequently', 'very frequently')) +
		xlab('Frequency of working with plants')
}

quest_plant_like = function(file) {
	q <- read.csv(file, header = TRUE)

	ggplot(
		data = q,
		aes(plant_like, ..count..)
	) +
		geom_bar() +
		scale_x_continuous(limits = c(-2.5, 2.5), breaks = seq(from = -2, to = 2, by = 1), labels = c('hate', 'dislike', 'neutral', 'like', 'love')) +
		xlab('Like plants')
}

quest_video_game = function(file) {
	q <- read.csv(file, header = TRUE)

	ggplot(
		data = q,
		aes(video_game, ..count..)
	) +
		geom_bar() +
		scale_x_continuous(limits = c(-2.5, 2.5), breaks = seq(from = -2, to = 2, by = 1), labels = c('never', 'rarely', 'occationally', 'frequently', 'very frequently')) +
		xlab('Frequency of playing video games')
}

quest_agree = function(file) {
	q <- read.csv(file, header = TRUE)

	ggplot(
		data = q,
		aes(ranking_agree, ..count..)
	) +
		geom_bar() +
		scale_x_continuous(limits = c(-2.5, 2.5), breaks = seq(from = -2, to = 2, by = 1), labels = c('strongly disagree', 'disagree', 'neutral', 'agree', 'strongly agree')) +
		xlab('Agree with fitness ranking')
}
