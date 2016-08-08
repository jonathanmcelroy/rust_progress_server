(function() {
    angular.module('progressServer').controller('programController', ['$state', '$resource', function(state, resource) {
        var programController = this;

        resource("/api/program/" + encodeURIComponent(state.params.name)).get(function(res) {
            programController.value = res.contents;
        })

        programController.value = "This is a value";
    }]);
}());
