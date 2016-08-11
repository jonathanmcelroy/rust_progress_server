(function() {
    angular.module('progressServer').controller('homeController', ['$state', function(state) {
        var homeController = this;
        homeController.searchText = "";

        homeController.onSearch = function() {
            state.go('search', {'contents': homeController.searchText});
        }

    }]);
}());
