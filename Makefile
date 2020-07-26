###############################################################################
#
# archive
#
###############################################################################

archive:
	node scripts/archive_binary_version.js

###############################################################################
#
# publish
#
###############################################################################

publish:
	bash $(MONOREPO_ROOT)/deploy/scripts/push-from-monorepo.sh \
		monorepo ridleywinters git@github.com:ridleywinters/raiment