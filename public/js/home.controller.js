(function() {
    angular.module('progressServer').controller('homeController', ['$state', function(state) {
        var homeController = this;
        homeController.searchProcedureText = "";

        homeController.innerSearchProcedureText = "";
        homeController.innerSearchInnerProcedureText = "";

        homeController.onSearchProcedure = function() {
            state.go('searchProcedure', {'contents': homeController.searchProcedureText });
        }
        homeController.onSearchInnerProcedure = function() {
            state.go('searchInnerProcedure', {
              'fileName': homeController.innerSearchProcedureText,
              'innerName': homeController.innerSearchInnerProcedureText,
            });
        }

    }]);
}());
